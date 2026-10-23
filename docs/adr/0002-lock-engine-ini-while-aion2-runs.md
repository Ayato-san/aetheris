# Lock Engine.ini editing while Aion 2 runs

Aetheris shows whether the Aion 2 client is running in the global title bar. The Engine.ini page remains readable, but editing and saving are disabled while the client is running; the write path checks again immediately before saving. If process status cannot be determined, Aetheris treats it as locked so the game cannot race with an edit.
