// util.cpp : ユーティリティ関数とか
//

#include "stdafx.h"
#include "ctx2pasta.h"

static std::map<duk_context*, pasta::Agent*> hashCtx2Pasta;

// duk_contextに対応するpastaエージェントへの参照を登録する。
void pasta::SetPasta(duk_context*ctx, pasta::Agent* pasta){
    hashCtx2Pasta[ctx] = pasta;
}

// duk_contextに対応するpastaエージェントへの参照を取得する。
pasta::Agent* pasta::GetPasta(duk_context*ctx){
    return  hashCtx2Pasta[ctx];
}