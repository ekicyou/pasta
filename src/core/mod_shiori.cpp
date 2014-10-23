// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"



// SHIORI ���X�|���X������Ԃ��܂��B
static duk_ret_t response(duk_context *ctx, pasta::Agent* pasta){
    USES_CONVERSION;
    OutputDebugString(L"[pasta::Shiori::response]�J�n�I\n");

    // ����
    auto res = A2CW_UTF8(duk_to_string(ctx, 0));
    pasta->Response(res);

    return 1;
}




void pasta::Agent::InitShiori(){

    auto module = "Shiori";
    auto &funcs = ShioriFuncs;

    funcs.push_back(Func(this, "response", response, 1));

    RegModule(module, funcs);
}


