"""Minimal obs-websocket v5 wire client for the Tier 4 black-box suite.

Wire-level only: speaks the documented v5 protocol (op codes 0-9), performs the
Hello/Identify handshake (optionally with password authentication), correlates
request/batch responses by requestId and collects events. No implementation
knowledge of the server is used beyond the published protocol.
"""

import asyncio
import base64
import hashlib
import json
import uuid

import websockets

# WebSocket close codes (v5 WebSocketCloseCode)
CLOSE_MESSAGE_DECODE_ERROR = 4002
CLOSE_MISSING_DATA_FIELD = 4003
CLOSE_INVALID_DATA_FIELD_TYPE = 4004
CLOSE_INVALID_DATA_FIELD_VALUE = 4005
CLOSE_UNKNOWN_OP_CODE = 4006
CLOSE_NOT_IDENTIFIED = 4007
CLOSE_ALREADY_IDENTIFIED = 4008
CLOSE_AUTHENTICATION_FAILED = 4009
CLOSE_UNSUPPORTED_RPC_VERSION = 4010
CLOSE_SESSION_INVALIDATED = 4011

# RequestStatus codes (v5)
STATUS_SUCCESS = 100
STATUS_MISSING_REQUEST_TYPE = 203
STATUS_UNKNOWN_REQUEST_TYPE = 204
STATUS_UNSUPPORTED_BATCH_EXECUTION_TYPE = 206
STATUS_MISSING_REQUEST_FIELD = 300
STATUS_MISSING_REQUEST_DATA = 301
STATUS_INVALID_REQUEST_FIELD = 400
STATUS_INVALID_REQUEST_FIELD_TYPE = 401
STATUS_REQUEST_FIELD_OUT_OF_RANGE = 402
STATUS_REQUEST_FIELD_EMPTY = 403
STATUS_TOO_MANY_REQUEST_FIELDS = 404
STATUS_OUTPUT_RUNNING = 500
STATUS_OUTPUT_NOT_RUNNING = 501
STATUS_OUTPUT_PAUSED = 502
STATUS_OUTPUT_NOT_PAUSED = 503
STATUS_STUDIO_MODE_ACTIVE = 505
STATUS_STUDIO_MODE_NOT_ACTIVE = 506
STATUS_RESOURCE_NOT_FOUND = 600
STATUS_RESOURCE_ALREADY_EXISTS = 601
STATUS_INVALID_RESOURCE_TYPE = 602
STATUS_INVALID_RESOURCE_STATE = 604
STATUS_INVALID_INPUT_KIND = 605
STATUS_RESOURCE_NOT_CONFIGURABLE = 606
STATUS_INVALID_FILTER_KIND = 607
STATUS_RESOURCE_CREATION_FAILED = 700
STATUS_RESOURCE_ACTION_FAILED = 701
STATUS_REQUEST_PROCESSING_FAILED = 702
STATUS_CANNOT_ACT = 703

# EventSubscription bits (v5)
SUB_NONE = 0
SUB_GENERAL = 1 << 0
SUB_CONFIG = 1 << 1
SUB_SCENES = 1 << 2
SUB_INPUTS = 1 << 3
SUB_TRANSITIONS = 1 << 4
SUB_FILTERS = 1 << 5
SUB_OUTPUTS = 1 << 6
SUB_SCENE_ITEMS = 1 << 7
SUB_MEDIA_INPUTS = 1 << 8
SUB_VENDORS = 1 << 9
SUB_UI = 1 << 10
SUB_CANVASES = 1 << 11
SUB_ALL = (
    SUB_GENERAL | SUB_CONFIG | SUB_SCENES | SUB_INPUTS | SUB_TRANSITIONS
    | SUB_FILTERS | SUB_OUTPUTS | SUB_SCENE_ITEMS | SUB_MEDIA_INPUTS
    | SUB_VENDORS | SUB_UI | SUB_CANVASES
)


def _auth_string(password: str, salt: str, challenge: str) -> str:
    """v5 authentication: base64(sha256(base64(sha256(password+salt))+challenge))."""
    secret = base64.b64encode(
        hashlib.sha256((password + salt).encode("utf-8")).digest()
    ).decode("utf-8")
    return base64.b64encode(
        hashlib.sha256((secret + challenge).encode("utf-8")).digest()
    ).decode("utf-8")


class CloseInfo:
    """Captured websocket close frame details."""

    def __init__(self, code=None, reason=""):
        self.code = code
        self.reason = reason

    def __repr__(self):
        return f"CloseInfo(code={self.code}, reason={self.reason!r})"


class ObsWsClient:
    """One websocket connection to an obs-websocket v5 server.

    A single pump task is the only consumer on the socket: it routes every
    incoming message into typed mailboxes (_by_op, _responses, _events) that
    the helpers then wait on.
    """

    def __init__(self, uri: str):
        self.uri = uri
        self.ws = None
        self.close = CloseInfo()
        self._responses: dict = {}
        self._events: list = []
        self._by_op: dict = {}
        self._pump = None

    async def connect(self):
        self.ws = await websockets.connect(self.uri, max_size=16 * 1024 * 1024)
        self._pump = asyncio.create_task(self._pump_loop())
        return await self.wait_op(0)

    async def _pump_loop(self):
        try:
            async for raw in self.ws:
                try:
                    msg = json.loads(raw)
                except (json.JSONDecodeError, TypeError):
                    continue
                op = msg.get("op")
                d = msg.get("d") or {}
                if op in (7, 9) and d.get("requestId") is not None:
                    self._responses[d["requestId"]] = msg
                elif op == 5:
                    self._events.append(msg)
                elif op is not None:
                    self._by_op.setdefault(op, []).append(msg)
        except websockets.exceptions.ConnectionClosed as e:
            self.close.code = e.code
            self.close.reason = e.reason or ""

    async def wait_op(self, op, timeout=10.0, index=0):
        """Wait for the next message with the given op code."""
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            msgs = self._by_op.get(op, [])
            if len(msgs) > index:
                return msgs[index]
            if self.ws is not None and self.ws.close_code is not None:
                return None
            await asyncio.sleep(0.01)
        return None

    async def identify(self, rpc_version=1, event_subscriptions=SUB_ALL,
                       password=None, auth_challenge=None, raw_d=None):
        """Send op=1 Identify. `raw_d` overrides the whole d payload (negative tests)."""
        if raw_d is not None:
            payload_d = raw_d
        else:
            payload_d = {
                "rpcVersion": rpc_version,
                "eventSubscriptions": event_subscriptions,
            }
            if password is not None and auth_challenge is not None:
                payload_d["authentication"] = _auth_string(
                    password,
                    auth_challenge["salt"],
                    auth_challenge["challenge"],
                )
        await self.ws.send(json.dumps({"op": 1, "d": payload_d}))

    async def request(self, request_type, request_data=None, request_id=None,
                      timeout=15.0):
        """Send op=6 Request and wait for op=7 RequestResponse."""
        rid = request_id or f"t4-{uuid.uuid4().hex[:8]}"
        d = {"requestType": request_type, "requestId": rid}
        if request_data is not None:
            d["requestData"] = request_data
        await self.ws.send(json.dumps({"op": 6, "d": d}))
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            if rid in self._responses:
                return self._responses.pop(rid)
            await asyncio.sleep(0.01)
        raise TimeoutError(f"no response for {request_type} ({rid})")

    async def request_batch(self, requests, execution_type=0,
                            halt_on_failure=False, request_id=None,
                            timeout=20.0):
        """Send op=8 RequestBatch, wait for op=9 RequestBatchResponse."""
        rid = request_id or f"t4b-{uuid.uuid4().hex[:8]}"
        d = {
            "requestId": rid,
            "haltOnFailure": halt_on_failure,
            "executionType": execution_type,
            "requests": requests,
        }
        await self.ws.send(json.dumps({"op": 8, "d": d}))
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            if rid in self._responses:
                return self._responses.pop(rid)
            await asyncio.sleep(0.01)
        raise TimeoutError(f"no batch response ({rid})")

    async def send_raw(self, payload):
        """Send a verbatim payload (str or already-dumped). For malformed tests."""
        await self.ws.send(payload if isinstance(payload, str) else json.dumps(payload))

    def events(self, event_type=None):
        """Snapshot of received op=5 events, optionally filtered by eventType."""
        if event_type is None:
            return list(self._events)
        return [e for e in self._events if e.get("d", {}).get("eventType") == event_type]

    async def wait_for_event(self, event_type, timeout=10.0):
        """Poll the received-events list until eventType appears or timeout."""
        deadline = asyncio.get_event_loop().time() + timeout
        while asyncio.get_event_loop().time() < deadline:
            hits = self.events(event_type)
            if hits:
                return hits[0]
            await asyncio.sleep(0.05)
        return None

    async def close_conn(self):
        if self.ws is not None:
            try:
                await self.ws.close()
            except Exception:
                pass
        if self._pump is not None:
            try:
                await asyncio.wait_for(self._pump, 2)
            except (asyncio.TimeoutError, asyncio.CancelledError):
                self._pump.cancel()
        if self.ws is not None:
            self.close.code = self.ws.close_code
            self.close.reason = self.ws.close_reason or ""
