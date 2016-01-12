start = lines
lines = l1:line l2:line2* { return [l1].concat(l2); }

line2 = BR l:line { return l;}



line = comment_line
     / element_line

comment_line = PRI_COMMENT comment:(NOT_BR*) &BR?
element_line = b:blank e:element* { return [b].concat(e);}
blank = s:SP* {return s.length == 0 ? "*": "B"; }


element = text
        / keyword



text = text:(esc / normal)+ { return "T" + text.join(""); }

esc    = &(KEY_MARK KEY_MARK) . mark:. { return [mark]; }
normal = chars:NORMAL_CHAR+ { return chars.join(""); }



keyword = keyword_mark body:keyword_body split { return body; }

keyword_mark = &KEY_MARK .
split        = SP+
             / &BR

keyword_body = keyword_local_jump
             / keyword_local_anchor
             / keyword_normal

keyword_normal       =                        key:KEY_CHAR+ { return "@" + key.join(""); }
keyword_local_jump   = (&LOCAL_JUMP_CHAR  ) . key:KEY_CHAR+ { return "J" + key.join(""); }
keyword_local_anchor = (&LOCAL_ANCHOR_CHAR) . key:KEY_CHAR+ { return "A" + key.join(""); }




BR          = "\r"? "\n"
SP          = [ 　]
KEY_MARK    = [@＠\\￥]
KEY_CHAR    = [^@＠\\￥ 　\r\n]
PRI_COMMENT = [#＃]
NOT_BR      = [^\r\n]
NORMAL_CHAR = [^@＠\\￥\r\n]
LOCAL_JUMP_CHAR   = [、,，]
LOCAL_ANCHOR_CHAR = [－ー-]

