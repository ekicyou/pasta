// mod_file_io.cpp : �g�ݍ��݊֐��Ffile�A�N�Z�X�֌W
//

#include "stdafx.h"
#include "agent_pasta.h"
#include "util.h"



static duk_ret_t loadModuleFile(duk_context *ctx, pasta::Agent* pasta){
    throw std::exception("not_impl");
}



void pasta::Agent::InitFileIO(){


    auto x = PastaFunc(this, "loadModuleFile", loadModuleFile, 2);
    


}
