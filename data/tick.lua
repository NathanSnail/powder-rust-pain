-- demo fps display, just showing it can do everything rust can.

st = st or GetTime() -- im not going to write a vscode language server for these funcs so their syntax wont get highlighted.
print("fps: " .. tostring(GetFrame() / (GetTime() - st) * 1000))

-- deltas = {{0,"data","fish"}} deltas can be manually handled in here, if you are a bit crazy