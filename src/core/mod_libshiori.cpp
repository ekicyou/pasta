// mod_shiorilib.cpp : �g�ݍ��݊֐��Fshiori�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"

//-------------------------------------------------------------
// SHIORI ���X�|���X������Ԃ��܂��B
static duk_ret_t response(duk_context *ctx){
    // �������������擾
    auto ghost = pasta::GetPasta(ctx);
    FUNC_START(ghost->cp);
    auto res = duk_to_string(ctx, 0);

    // GET���X�|���X���s�B
    ghost->Response(res);

    return 1;
}

//-------------------------------------------------------------
// �f�o�b�O�o�͂��s���܂��B
static duk_ret_t debugstring(duk_context *ctx){
    // �������������擾
    auto ghost = pasta::GetPasta(ctx);
    auto cp = ghost->cp;
#ifdef DEBUG
    std::string text(duk_to_string(ctx, 0));
    text += "\n";
    USES_CONVERSION;
    OutputDebugString(A2CW_CP(text.c_str(), cp));
#endif

    return 1;
}

//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { "debugstring", debugstring, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitModuleShiori(){
    RegModuleFuncs("libshiori", funcs);
}

// EOF