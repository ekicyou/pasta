/*
 *  Unicode support tables automatically generated during build.
 */

#include "duk_internal.h"

/*
 *  Unicode tables containing ranges of Unicode characters in a
 *  packed format.  These tables are used to match non-ASCII
 *  characters of complex productions by resorting to a linear
 *  range-by-range comparison.  This is very slow, but is expected
 *  to be very rare in practical Ecmascript source code, and thus
 *  compactness is most important.
 *
 *  The tables are matched using uni_range_match() and the format
 *  is described in src/extract_chars.py.
 */

#ifdef DUK_USE_SOURCE_NONBMP
/* IdentifierStart production with ASCII excluded */
/* duk_unicode_ids_noa[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_ids_noa[797] = {
249,176,176,80,111,7,47,15,47,254,11,197,191,0,72,2,15,115,66,19,57,2,34,2,
240,66,244,50,247,185,248,234,241,99,8,241,127,58,240,182,47,31,241,191,21,
18,245,50,15,1,24,27,35,15,2,2,240,239,15,244,156,15,10,241,26,21,6,240,
101,10,4,15,9,240,159,157,242,100,15,4,8,159,1,98,102,115,19,240,98,98,4,
52,15,2,14,18,47,0,31,5,85,19,240,98,98,18,18,31,17,50,15,5,47,2,130,34,
240,98,98,18,68,15,4,15,1,31,21,115,19,240,98,98,18,68,15,16,18,47,1,15,3,
2,84,34,52,18,2,20,20,36,191,8,15,38,114,34,240,114,146,68,15,12,23,31,21,
114,34,240,114,146,68,15,18,2,31,1,31,4,114,34,241,147,15,2,15,3,31,10,86,
240,36,240,130,130,3,111,44,242,2,29,111,44,18,3,18,3,7,50,98,34,2,3,18,50,
26,3,66,15,7,31,20,15,49,114,241,79,13,79,101,241,191,6,15,2,85,52,4,24,37,
205,15,3,241,107,241,178,4,255,224,59,35,54,32,35,63,25,35,63,17,35,54,32,
35,62,47,41,35,63,51,241,127,0,240,47,69,223,254,21,227,240,18,240,166,243,
180,47,1,194,63,0,240,47,0,240,47,0,194,47,1,242,79,21,5,15,53,244,137,241,
146,6,243,107,240,223,37,240,227,76,241,207,7,111,42,240,122,242,95,68,15,
79,241,255,3,111,41,240,238,31,2,241,111,12,241,79,27,43,241,79,93,50,63,0,
251,15,50,255,224,8,53,63,22,53,55,32,32,32,47,15,63,37,38,32,66,38,67,53,
92,98,38,246,96,224,240,44,245,112,80,57,32,68,112,32,32,35,42,51,100,80,
240,63,25,255,233,107,241,242,241,242,247,87,63,3,241,107,242,106,15,2,240,
122,98,98,98,98,98,98,98,111,66,15,254,12,146,240,184,132,52,95,70,114,47,
74,35,111,25,79,78,240,63,11,242,127,0,255,224,244,15,255,0,8,168,15,60,15,
255,0,64,190,15,38,255,227,127,243,95,30,63,253,79,0,177,240,111,31,240,47,
9,159,64,241,152,63,87,51,33,240,9,244,39,34,35,47,7,240,255,36,240,15,34,
243,5,64,240,15,12,191,7,240,191,13,143,31,240,224,242,47,25,240,146,39,
240,111,7,64,111,32,32,65,52,48,32,240,162,241,85,53,53,166,38,248,63,19,
240,240,255,240,1,169,96,223,7,95,33,255,240,0,255,143,254,2,3,242,227,245,
175,24,109,70,2,146,194,66,2,18,18,245,207,19,255,224,93,240,79,48,63,38,
241,171,246,100,47,119,241,111,10,127,10,207,73,69,53,53,50,241,91,47,10,
47,3,33,46,61,241,79,107,243,127,37,255,223,13,79,33,242,31,15,240,63,11,
242,127,14,63,20,87,36,241,207,142,255,226,86,83,2,241,194,20,3,240,127,
156,240,107,240,175,184,15,1,50,34,240,191,30,240,223,117,242,107,240,107,
240,63,127,243,159,254,42,239,37,243,223,29,255,238,68,255,226,97,248,63,
83,255,234,145,255,227,33,255,240,2,44,95,254,18,191,255,0,52,187,31,255,0,
18,242,244,82,243,114,19,3,19,50,178,2,98,243,18,51,114,98,240,194,50,66,4,
98,255,224,70,63,9,47,9,47,15,47,9,47,15,47,9,47,15,47,9,47,15,47,9,39,255,
240,1,114,128,255,240,9,92,144,241,176,255,239,39,12,15,206,15,255,0,46,
214,255,225,16,0,
};
#else
/* IdentifierStart production with ASCII and non-BMP excluded */
/* duk_unicode_ids_noabmp[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_ids_noabmp[614] = {
249,176,176,80,111,7,47,15,47,254,11,197,191,0,72,2,15,115,66,19,57,2,34,2,
240,66,244,50,247,185,248,234,241,99,8,241,127,58,240,182,47,31,241,191,21,
18,245,50,15,1,24,27,35,15,2,2,240,239,15,244,156,15,10,241,26,21,6,240,
101,10,4,15,9,240,159,157,242,100,15,4,8,159,1,98,102,115,19,240,98,98,4,
52,15,2,14,18,47,0,31,5,85,19,240,98,98,18,18,31,17,50,15,5,47,2,130,34,
240,98,98,18,68,15,4,15,1,31,21,115,19,240,98,98,18,68,15,16,18,47,1,15,3,
2,84,34,52,18,2,20,20,36,191,8,15,38,114,34,240,114,146,68,15,12,23,31,21,
114,34,240,114,146,68,15,18,2,31,1,31,4,114,34,241,147,15,2,15,3,31,10,86,
240,36,240,130,130,3,111,44,242,2,29,111,44,18,3,18,3,7,50,98,34,2,3,18,50,
26,3,66,15,7,31,20,15,49,114,241,79,13,79,101,241,191,6,15,2,85,52,4,24,37,
205,15,3,241,107,241,178,4,255,224,59,35,54,32,35,63,25,35,63,17,35,54,32,
35,62,47,41,35,63,51,241,127,0,240,47,69,223,254,21,227,240,18,240,166,243,
180,47,1,194,63,0,240,47,0,240,47,0,194,47,1,242,79,21,5,15,53,244,137,241,
146,6,243,107,240,223,37,240,227,76,241,207,7,111,42,240,122,242,95,68,15,
79,241,255,3,111,41,240,238,31,2,241,111,12,241,79,27,43,241,79,93,50,63,0,
251,15,50,255,224,8,53,63,22,53,55,32,32,32,47,15,63,37,38,32,66,38,67,53,
92,98,38,246,96,224,240,44,245,112,80,57,32,68,112,32,32,35,42,51,100,80,
240,63,25,255,233,107,241,242,241,242,247,87,63,3,241,107,242,106,15,2,240,
122,98,98,98,98,98,98,98,111,66,15,254,12,146,240,184,132,52,95,70,114,47,
74,35,111,25,79,78,240,63,11,242,127,0,255,224,244,15,255,0,8,168,15,60,15,
255,0,64,190,15,38,255,227,127,243,95,30,63,253,79,0,177,240,111,31,240,47,
9,159,64,241,152,63,87,51,33,240,9,244,39,34,35,47,7,240,255,36,240,15,34,
243,5,64,240,15,12,191,7,240,191,13,143,31,240,224,242,47,25,240,146,39,
240,111,7,64,111,32,32,65,52,48,32,240,162,241,85,53,53,166,38,248,63,19,
240,240,255,240,1,169,96,223,7,95,33,255,240,0,255,143,254,2,3,242,227,245,
175,24,109,70,2,146,194,66,2,18,18,245,207,19,255,224,93,240,79,48,63,38,
241,171,246,100,47,119,241,111,10,127,10,207,73,69,53,53,50,0,
};
#endif

#ifdef DUK_USE_SOURCE_NONBMP
/* IdentifierStart production with Letter and ASCII excluded */
/* duk_unicode_ids_m_let_noa[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_ids_m_let_noa[42] = {
255,240,0,94,18,255,233,99,241,51,63,254,215,32,240,184,240,2,255,240,6,89,
249,255,240,4,148,79,37,255,224,192,9,15,120,79,255,0,15,30,245,48,
};
#else
/* IdentifierStart production with Letter, ASCII, and non-BMP excluded */
/* duk_unicode_ids_m_let_noabmp[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_ids_m_let_noabmp[24] = {
255,240,0,94,18,255,233,99,241,51,63,254,215,32,240,184,240,2,255,240,6,89,
249,0,
};
#endif

#ifdef DUK_USE_SOURCE_NONBMP
/* IdentifierPart production with IdentifierStart and ASCII excluded */
/* duk_unicode_idp_m_ids_noa[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_idp_m_ids_noa[397] = {
255,225,243,246,15,254,0,116,255,191,29,32,33,33,32,243,170,242,47,15,112,
245,118,53,49,35,57,240,144,241,15,11,244,218,240,25,241,56,241,67,40,34,
36,241,210,249,99,242,130,47,2,38,177,57,240,50,242,160,38,49,50,160,177,
57,240,50,242,160,36,81,50,64,240,107,64,194,242,160,39,34,34,240,97,57,
240,50,242,160,38,49,50,145,177,57,240,64,242,212,66,35,160,240,9,240,50,
242,198,34,35,129,193,57,240,65,242,160,38,34,35,129,193,57,240,65,242,198,
34,35,160,177,57,240,65,243,128,85,32,39,240,65,242,240,54,215,41,244,144,
53,33,197,57,243,1,121,192,32,32,81,242,63,4,33,106,47,20,160,245,111,4,41,
211,82,34,54,67,235,46,255,225,179,47,254,42,98,240,242,240,241,241,1,243,
79,14,160,57,241,50,57,248,16,246,139,91,185,245,47,1,129,121,242,244,242,
185,47,13,58,121,245,132,242,31,1,201,240,56,210,241,9,105,241,237,242,47,
4,153,121,246,130,47,5,80,80,251,255,23,240,115,255,225,0,31,35,31,5,15,
109,197,4,191,254,175,34,247,240,245,47,16,255,225,30,95,91,31,255,0,100,
121,159,55,13,31,100,31,254,0,64,64,80,240,148,244,161,242,79,1,201,127,2,
240,9,240,231,240,188,241,227,242,29,240,25,244,29,208,145,57,241,48,242,
96,34,49,97,32,255,224,21,114,19,159,255,0,62,24,15,254,29,95,0,240,38,209,
240,162,251,41,241,112,255,225,177,15,254,25,105,255,228,75,34,22,63,26,37,
15,254,75,66,242,126,241,25,240,34,241,250,255,240,10,249,228,69,151,54,
241,3,248,98,255,228,125,242,47,255,12,23,244,254,0,
};
#else
/* IdentifierPart production with IdentifierStart, ASCII, and non-BMP excluded */
/* duk_unicode_idp_m_ids_noabmp[] */
/*
 *  Automatically generated by extract_chars.py, do not edit!
 */

const duk_uint8_t duk_unicode_idp_m_ids_noabmp[348] = {
255,225,243,246,15,254,0,116,255,191,29,32,33,33,32,243,170,242,47,15,112,
245,118,53,49,35,57,240,144,241,15,11,244,218,240,25,241,56,241,67,40,34,
36,241,210,249,99,242,130,47,2,38,177,57,240,50,242,160,38,49,50,160,177,
57,240,50,242,160,36,81,50,64,240,107,64,194,242,160,39,34,34,240,97,57,
240,50,242,160,38,49,50,145,177,57,240,64,242,212,66,35,160,240,9,240,50,
242,198,34,35,129,193,57,240,65,242,160,38,34,35,129,193,57,240,65,242,198,
34,35,160,177,57,240,65,243,128,85,32,39,240,65,242,240,54,215,41,244,144,
53,33,197,57,243,1,121,192,32,32,81,242,63,4,33,106,47,20,160,245,111,4,41,
211,82,34,54,67,235,46,255,225,179,47,254,42,98,240,242,240,241,241,1,243,
79,14,160,57,241,50,57,248,16,246,139,91,185,245,47,1,129,121,242,244,242,
185,47,13,58,121,245,132,242,31,1,201,240,56,210,241,9,105,241,237,242,47,
4,153,121,246,130,47,5,80,80,251,255,23,240,115,255,225,0,31,35,31,5,15,
109,197,4,191,254,175,34,247,240,245,47,16,255,225,30,95,91,31,255,0,100,
121,159,55,13,31,100,31,254,0,64,64,80,240,148,244,161,242,79,1,201,127,2,
240,9,240,231,240,188,241,227,242,29,240,25,244,29,208,145,57,241,48,242,
96,34,49,97,32,255,224,21,114,19,159,255,0,62,24,15,254,29,95,0,240,38,209,
240,162,251,41,241,112,0,
};
#endif

/*
 *  Case conversion tables generated using src/extract_caseconv.py.
 */

/* duk_unicode_caseconv_uc[] */
/* duk_unicode_caseconv_lc[] */

/*
 *  Automatically generated by extract_caseconv.py, do not edit!
 */

const duk_uint8_t duk_unicode_caseconv_uc[1288] = {
132,3,128,3,0,184,7,192,6,192,112,35,242,199,224,64,74,192,49,32,128,162,
128,108,65,1,189,129,254,131,3,173,3,136,6,7,98,7,34,68,15,12,14,140,72,30,
104,28,112,32,67,0,65,4,0,138,0,128,4,1,88,65,76,83,15,128,15,132,8,31,16,
31,24,12,62,64,62,80,32,124,192,124,224,64,250,0,250,64,97,246,1,246,129,3,
238,3,247,64,135,220,135,242,2,15,187,15,237,2,31,120,31,248,4,62,244,63,
212,8,125,240,127,232,16,253,128,253,192,33,253,1,253,128,67,252,3,253,0,
136,92,8,88,8,18,104,18,91,26,44,48,44,0,94,90,0,33,64,155,253,7,252,132,
212,0,32,32,32,6,0,76,192,76,129,128,157,0,156,136,1,75,1,74,46,2,244,2,
242,12,6,12,6,8,16,13,8,13,0,48,27,64,27,48,64,57,192,57,162,0,119,192,119,
132,128,252,128,252,20,2,35,2,34,18,4,142,4,140,20,13,196,13,192,16,30,200,
30,192,192,70,16,70,2,32,145,96,145,70,193,48,129,48,67,130,104,130,104,44,
30,1,30,0,150,61,66,61,64,192,125,68,125,100,33,99,65,99,56,50,200,18,200,
6,69,157,133,157,96,169,144,105,144,11,211,64,211,64,12,167,35,167,34,15,
78,103,78,100,126,157,234,157,228,21,59,253,59,240,90,122,26,122,0,163,128,
214,128,214,2,1,197,1,196,6,3,140,3,136,12,7,200,7,196,16,20,0,13,48,32,63,
128,63,112,69,142,101,142,64,130,1,136,1,135,4,3,114,3,112,8,26,120,202,
120,176,65,1,30,1,29,130,2,105,1,150,5,255,96,22,160,115,128,31,224,47,0,
38,32,9,32,47,224,10,96,48,0,72,96,50,64,50,32,50,160,62,192,51,32,51,0,51,
64,71,160,51,192,68,0,53,0,52,224,55,224,62,224,59,160,49,192,62,96,62,32,
74,5,141,224,74,37,141,160,74,69,142,0,74,96,48,32,74,128,48,192,75,32,49,
224,75,96,50,0,76,0,50,96,76,96,50,128,76,180,241,160,77,0,50,224,77,101,
140,64,78,37,141,192,78,64,51,160,78,160,51,224,79,165,140,128,81,0,53,192,
81,32,72,128,81,128,72,160,82,64,54,224,104,160,115,32,110,224,110,192,117,
128,112,192,120,64,116,96,121,128,113,128,122,0,114,64,122,32,115,0,122,
160,116,192,122,192,116,0,122,224,121,224,126,0,115,64,126,32,116,32,126,
64,127,32,126,160,114,160,153,224,152,3,175,52,239,163,175,165,140,99,211,
99,204,3,247,192,115,35,252,163,253,132,41,196,38,68,48,132,48,101,140,37,
140,5,140,160,71,69,140,192,71,217,128,55,224,5,48,5,48,20,152,10,240,1,56,
7,194,0,74,3,12,3,144,192,230,64,194,0,192,64,236,48,58,80,48,128,48,16,88,
120,20,212,21,72,122,90,0,72,3,49,30,151,128,21,0,194,7,166,32,5,112,48,
161,233,152,1,100,12,40,122,106,0,65,2,190,31,80,128,233,64,196,199,212,
176,58,80,49,48,48,1,245,76,14,148,12,76,12,4,125,91,3,165,3,19,3,66,31,
128,135,194,0,230,71,224,97,240,144,57,145,248,40,124,40,14,100,126,14,31,
11,3,153,31,132,135,195,0,230,71,225,97,240,208,57,145,248,104,124,56,14,
100,126,30,31,15,3,153,31,136,135,194,0,230,71,226,97,240,144,57,145,248,
168,124,40,14,100,126,46,31,11,3,153,31,140,135,195,0,230,71,227,97,240,
208,57,145,248,232,124,56,14,100,126,62,31,15,3,153,31,144,135,202,0,230,
71,228,97,242,144,57,145,249,40,124,168,14,100,126,78,31,43,3,153,31,148,
135,203,0,230,71,229,97,242,208,57,145,249,104,124,184,14,100,126,94,31,47,
3,153,31,152,135,202,0,230,71,230,97,242,144,57,145,249,168,124,168,14,100,
126,110,31,43,3,153,31,156,135,203,0,230,71,231,97,242,208,57,145,249,232,
124,184,14,100,126,126,31,47,3,153,31,160,135,218,0,230,71,232,97,246,144,
57,145,250,40,125,168,14,100,126,142,31,107,3,153,31,164,135,219,0,230,71,
233,97,246,208,57,145,250,104,125,184,14,100,126,158,31,111,3,153,31,168,
135,218,0,230,71,234,97,246,144,57,145,250,168,125,168,14,100,126,174,31,
107,3,153,31,172,135,219,0,230,71,235,97,246,208,57,145,250,232,125,184,14,
100,126,190,31,111,3,153,31,178,135,238,128,230,71,236,224,57,16,57,145,
251,72,14,24,14,100,126,218,3,145,3,66,31,183,192,228,64,208,128,230,71,
239,32,57,16,57,145,252,40,127,40,14,100,127,14,3,151,3,153,31,196,128,226,
64,230,71,241,160,57,112,52,33,252,124,14,92,13,8,14,100,127,50,3,151,3,
153,31,210,192,230,64,194,0,192,7,244,240,57,144,48,128,48,17,253,104,14,
100,13,8,127,95,3,153,3,8,3,66,31,226,192,233,64,194,0,192,7,248,240,58,80,
48,128,48,17,254,72,14,132,12,76,127,154,3,165,3,66,31,231,192,233,64,194,
0,208,135,252,161,255,160,57,145,255,56,14,164,14,100,127,210,3,143,3,153,
31,246,128,234,64,208,135,253,240,58,144,52,32,57,145,255,200,14,164,14,
103,236,2,0,70,0,70,251,1,128,17,128,18,126,192,160,4,96,4,207,176,60,1,24,
1,24,1,39,236,19,0,70,0,70,0,76,251,5,128,20,192,21,62,193,160,5,48,5,79,
177,56,21,16,21,27,236,82,5,68,5,53,251,21,129,81,1,78,254,197,160,84,224,
84,111,177,120,21,16,20,244,
};
const duk_uint8_t duk_unicode_caseconv_lc[616] = {
144,3,0,3,128,184,6,192,7,192,112,24,144,37,96,64,54,32,81,64,128,226,0,
235,65,129,199,1,230,130,3,145,3,177,34,7,70,7,134,36,15,244,13,236,24,32,
0,34,129,0,65,0,67,4,0,166,32,172,41,132,40,11,64,19,15,132,15,128,8,31,24,
31,16,12,62,80,62,64,32,124,224,124,192,64,250,64,250,0,97,246,129,246,1,3,
241,3,240,2,7,230,7,228,4,15,212,15,208,8,31,184,31,176,4,63,116,62,224,8,
127,32,125,200,32,254,192,254,128,33,253,161,247,96,67,253,3,252,0,135,250,
135,222,129,15,252,15,188,2,31,250,31,124,4,66,192,66,224,64,146,216,147,
64,209,96,1,97,130,242,199,224,35,240,95,228,63,232,38,161,1,0,1,1,48,2,
100,2,102,12,4,228,4,232,64,10,80,10,89,112,23,144,23,160,96,48,64,48,96,
128,104,0,104,65,128,217,128,218,2,1,203,1,204,18,3,188,3,190,36,7,200,7,
204,16,15,192,15,201,64,34,32,34,49,32,72,192,72,225,64,220,0,220,65,1,236,
1,236,140,4,96,4,97,34,9,20,9,22,108,19,4,19,8,56,38,128,38,138,193,224,1,
224,25,99,212,3,212,44,7,214,71,212,66,22,51,150,52,3,44,128,44,129,100,89,
214,89,216,10,153,2,153,4,189,52,5,52,8,202,114,42,114,48,244,230,84,230,
103,233,222,105,222,129,83,191,83,191,133,167,160,167,161,10,48,13,48,20,0,
32,26,192,26,208,64,56,128,56,192,192,113,64,113,129,1,251,129,252,2,44,
114,44,115,4,16,12,56,12,64,32,27,128,27,144,64,211,197,211,198,2,8,6,88,9,
164,16,17,216,17,224,47,245,1,120,0,255,1,129,2,83,1,134,2,84,1,142,1,221,
1,143,2,89,1,144,2,91,1,145,1,146,1,147,2,96,1,148,2,99,1,151,2,104,1,152,
1,153,1,157,2,114,1,159,2,117,1,167,1,168,1,174,2,136,1,183,2,146,1,241,1,
243,1,246,1,149,1,247,1,191,2,32,1,158,2,58,44,101,2,61,1,154,2,62,44,102,
2,67,1,128,2,68,2,137,2,69,2,140,3,118,3,119,3,134,3,172,3,140,3,204,3,207,
3,215,3,244,3,184,3,249,3,242,4,192,4,207,30,158,0,223,31,188,31,179,31,
204,31,195,31,236,31,229,31,252,31,243,33,38,3,201,33,42,0,107,33,43,0,229,
33,50,33,78,33,131,33,132,44,96,44,97,44,98,2,107,44,99,29,125,44,100,2,
125,44,109,2,81,44,110,2,113,44,111,2,80,44,112,2,82,167,125,29,121,167,
141,2,101,2,2,97,0,52,129,131,128,
};
