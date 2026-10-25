# Exhume never reads real `.env` files

Env vars are only ever checked against env documentation (`.env.example`, `.env.*.example`, `.env.sample`, Dockerfiles, compose files, CI workflows, `turbo.json`), never against `.env`, `.env.local` or other files that hold values. Those files contain secrets. Exhume's output goes into CI logs, JSON artifacts and public PR descriptions, so a tool that never opens secret files cannot leak them. The cost: a var that is defined only in a local `.env` is reported as an Undocumented Env Var, and that is the intended behaviour.
