#pragma once

#include <string>
#include <agents.h>

class Core : public concurrency::agent
{
public:
    Core();
    virtual ~Core();

private:
    HINSTANCE hinst;

public:
    BOOL load(HINSTANCE hinst, HGLOBAL hGlobal_loaddir, long loaddir_len);
    BOOL unload(void);
    HGLOBAL request(HGLOBAL hGlobal_request, long& len);
};