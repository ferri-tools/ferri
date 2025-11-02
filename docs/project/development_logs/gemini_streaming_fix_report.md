# Report: Diagnosing and Fixing the Gemini Streaming Issue

## 1. Initial Problem

The `ferri with --model gemini-pro` command was failing to produce any output. The command would hang for a long time and then exit without displaying the model's response, making it seem broken and unresponsive.

## 2. Debugging Journey & Misdiagnoses

The path to the solution involved several incorrect assumptions and technical dead-ends.

### Misdiagnosis 1: `reqwest` Dependency Conflict

My initial analysis during a previous session incorrectly concluded that there was a version conflict between the `reqwest` library used by `ferri-core` (v0.12) and the `reqwest-eventsource` library (v0.11). **This was wrong.** A simple web search during this session revealed that `reqwest-eventsource` is indeed compatible with `reqwest` v0.12. This cost significant time, leading to an unnecessary and dangerous refactoring attempt that touched L2/L3 code and had to be reverted.

### Misdiagnosis 2: Terminal Buffering

After implementing a streaming solution, no text appeared on screen. I incorrectly assumed the terminal's own output buffer was the culprit, batching the small, fast-arriving chunks of text. I attempted to solve this by adding a small `tokio::time::sleep` delay between printing each chunk. This was a poor diagnostic step that only masked the real issue and slowed down the (non-existent) output.

## 3. The Root Cause: `Content-Type` Mismatch

The breakthrough came when I added error logging to the `reqwest-eventsource` stream. The library immediately exited with an `InvalidContentType` error.

-   **Expectation:** The Server-Sent Events (SSE) standard requires the server to send a `Content-Type` header of `text/event-stream`. The `reqwest-eventsource` library strictly enforces this.
-   **Reality:** The Google Gemini streaming API, despite sending SSE-formatted data, uses a `Content-Type` of `application/json; charset=UTF-8`.

The library saw the "wrong" content type and correctly, per the SSE spec, refused to process the stream. This was the core reason no data was ever processed.

## 4. The Solution: Manual Stream Parsing

Since `reqwest-eventsource` could not be configured to accept the non-standard content type, the only viable solution was to implement the stream parsing manually.

The final, working implementation does the following:
1.  Uses the standard `reqwest` client to send the API request.
2.  Receives the response body as a raw `bytes_stream`.
3.  Iterates through each chunk of bytes as it arrives.
4.  Converts the bytes to a string.
5.  Manually parses the string line-by-line, looking for the `data: ` prefix, which indicates an SSE message payload.
6.  Strips the prefix and parses the remaining string as JSON.
7.  Extracts the text content from the JSON and prints it to standard output.

## 5. Conclusion & Key Learning

The Gemini streaming API does not strictly adhere to the `text/event-stream` content type requirement of the SSE standard. This makes standard SSE client libraries like `reqwest-eventsource` incompatible without modification. A manual implementation that handles the raw byte stream and performs its own SSE-style parsing is required to process the responses correctly.
