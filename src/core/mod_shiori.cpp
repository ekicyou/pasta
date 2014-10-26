// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



// SHIORI ���X�|���X������Ԃ��܂��B
static duk_ret_t response(duk_context *ctx){
    FUNC_START;
    USES_CONVERSION;

    // �������������擾
    auto ghost = pasta::GetPasta(ctx);
    auto res = A2CW_UTF8(duk_to_string(ctx, 0));

    ghost->Response(res);

    return 1;
}


//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "response", response, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitShiori(){
    RegModuleFuncs("Shiori", funcs);
}

// EOF