// util.cpp : ���[�e�B���e�B�֐��Ƃ�
//

#include "stdafx.h"
#include "ctx2pasta.h"

static std::map<duk_context*, pasta::Agent*> hashCtx2Pasta;

// duk_context�ɑΉ�����pasta�G�[�W�F���g�ւ̎Q�Ƃ�o�^����B
void pasta::SetPasta(duk_context*ctx, pasta::Agent* pasta){
    hashCtx2Pasta[ctx] = pasta;
}

// duk_context�ɑΉ�����pasta�G�[�W�F���g�ւ̎Q�Ƃ��擾����B
pasta::Agent* pasta::GetPasta(duk_context*ctx){
    return  hashCtx2Pasta[ctx];
}