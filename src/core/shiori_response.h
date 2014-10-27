#pragma once

#define WSTR_RES_NO_CONTENT (                   \
    L"SHIORI/3.0 204 No Content\r\n"            \
    L"Charset: UTF-8\r\n"                       \
    L"\r\n"                                     \
    )

#define WSTR_RES_BAT_REQUEST (                  \
    L"SHIORI/3.0 400 Bad Request\r\n"           \
    L"Charset: UTF-8\r\n"                       \
    L"\r\n"                                     \
    )

#define WSTR_RES_SERVER_ERROR (                 \
    L"SHIORI/3.0 500 Internal Server Error\r\n" \
    L"Charset: UTF-8\r\n"                       \
    L"Sender: PASTA\r\n"                        \
    )

#define STR_RES_SERVER_ERROR (                  \
    "SHIORI/3.0 500 Internal Server Error\r\n"  \
    "Charset: UTF-8\r\n"                        \
    "Sender: PASTA\r\n"                         \
    )
