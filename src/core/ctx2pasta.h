#pragma once

#include "agent_pasta.h"
#include <duktape.h>

namespace pasta{
    // duk_contextに対応するpastaエージェントへの参照を登録する。
    void SetPasta(duk_context* ctx, pasta::Agent* pasta);

    // duk_contextに対応するpastaエージェントへの参照を取得する。
    Agent* GetPasta(duk_context* ctx);
}