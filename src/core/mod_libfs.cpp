// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "ctx2pasta.h"
#include "util.h"



//-------------------------------------------------------------
// �I�[�v�����ꂽFILE*���o�b�t�@�Ƃ��ăX�^�b�N�ɐςށB
static duk_ret_t push_file_to_buffer(duk_context *ctx, FILE* f){
    duk_ret_t rc = DUK_RET_ERROR;

    if (fseek(f, 0, SEEK_END) != 0) goto clean;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto clean;
    auto buf = duk_push_fixed_buffer(ctx, (size_t)len);
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len) goto clean;
    rc = 1;

clean:
    return rc;
}


//-------------------------------------------------------------
// �I�[�v�����ꂽFILE*�𕶎���Ƃ��ăX�^�b�N�ɐςށB
static duk_ret_t push_file_to_string(duk_context *ctx, FILE* f){
    duk_ret_t rc = DUK_RET_ERROR;
    char *buf = NULL;

    // �ǂݍ���
    if (fseek(f, 0, SEEK_END) != 0) goto clean;
    auto len = ftell(f);
    if (fseek(f, 0, SEEK_SET) != 0) goto clean;
    buf = (char *)malloc(len);
    if (!buf)                       goto clean;
    auto got = fread(buf, 1, len, f);
    if (got != (size_t)len)         goto clean;
    duk_push_lstring(ctx, buf, got);
    rc = 1;

clean:
    if (buf) free(buf);
    return rc;
}





//-------------------------------------------------------------
// �t�@�C�����o�b�t�@�Ƃ��ēǂݍ��݂܂��B
static duk_ret_t readfile(duk_context *ctx){
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // �������������擾
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);

    // �t�@�C���I�[�v�����ǂݍ���
    auto f = pasta->OpenReadModuleFile(fname);
    if (!f)goto clean;
    rc = push_file_to_buffer(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}


//-------------------------------------------------------------
// �t�@�C�����e�L�X�g�Ƃ��ēǂݍ��݂܂��B
static duk_ret_t readtext(duk_context *ctx){
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // �������������擾
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);

    // �t�@�C���I�[�v�����ǂݍ���
    auto f = pasta->OpenReadModuleFile(fname);
    if (!f)goto clean;
    rc = push_file_to_string(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}

//-------------------------------------------------------------
// ���[�U�[�t�@�C�����e�L�X�g�Ƃ��ēǂݍ��݂܂��B
static duk_ret_t readuser(duk_context *ctx){
    USES_CONVERSION;
    auto pasta = pasta::GetPasta(ctx);
    FUNC_START(pasta->cp);

    // �������������擾
    duk_ret_t rc = DUK_RET_ERROR;
    auto fname = duk_to_string(ctx, 0);
    char *buf = NULL;

    // �t�@�C���I�[�v��
    auto f = pasta->OpenReadUserFile(fname);
    if (!f)goto clean;
    rc = push_file_to_string(ctx, f);

clean:
    if (f) fclose(f);
    return rc;
}


//-------------------------------------------------------------
// ���W���[���o�^
//-------------------------------------------------------------

static duk_function_list_entry funcs[] = {
        { "readfile", readfile, 1 },
        { "readtext", readtext, 1 },
        { "readuser", readuser, 1 },
        { NULL, NULL, 0 }
};

void pasta::Agent::InitModuleFileIO(){
    RegModuleFuncs("libfs", funcs);
}

// EOF