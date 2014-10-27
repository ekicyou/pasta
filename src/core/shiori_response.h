#pragma once

#define STR_RES_NO_CONTENT (                    \
    "SHIORI/3.0 204 No Content\r\n"             \
    "Charset: UTF-8\r\n"                        \
    "\r\n"                                      \
    )

#define STR_RES_BAT_REQUEST (                   \
    "SHIORI/3.0 400 Bad Request\r\n"            \
    "Charset: UTF-8\r\n"                        \
    "\r\n"                                      \
    )

#define STR_RES_SERVER_ERROR (                  \
    "SHIORI/3.0 500 Internal Server Error\r\n"  \
    "Charset: UTF-8\r\n"                        \
    "Sender: PASTA\r\n"                         \
    )
