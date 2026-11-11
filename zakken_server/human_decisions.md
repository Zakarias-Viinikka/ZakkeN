In order to prevent race conditions for drift stuff:

Whoever receives a request has to check if the request is on the same
drift version as it's operating on currently. Otherwise it gets thrown
away.

Same applies when a client receives a love letter from a server
spontaneously. If the client was in the middle of starting a drift, then
the client needs to check "should I actually accept this or not".
---
how clients are informed they've drifted:

A client knows it has drifted from the server in one of two ways: an
error happens, and it enters drift deliberately, or the client enters
drift deliberately on its own. Either way it would know. And even if it
didn't, as long as the drift version changes, then once it logs in it
finds out it's on a different version.

If the server causes the drift (due to another client), the server
changes its drift version and disconnects any clients that would be
drifted. It returns an error on any requests made on old drift versions,
and it sends a spontaneous message to all the clients it disconnects
that tells them what happened. That's how clients know to try and
reconnect.

This way it's impossible for a client to be connected and on the wrong
version. Unless they cause the drift deliberately and don't disconnect,
but that doesn't cause any errors, because they'd be throwing away any
info they receive from the server, and if they tried to make the server
do stuff then the server would throw away their requests.

Logging in basically informs any client that they've drifted.
---
Changing the server drift version should queue up a "disconnect
everyone eventually" event, and it only fires once all the clients have
been disconnected. The thing that enforces it, is refusing new
connections, and the telephone sending a "disconnect me" to every
socket.

reconcile_version is drift_version but for reconciles because otherwise there isn't a way to prevent two reconciles happening at the same time. Allowing that would break stuff in a nasty way even though it's unlikely.
