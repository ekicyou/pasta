﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    public class RequiredIfNotRegExMatchAttribute : RequiredIfAttribute
    {
        public RequiredIfNotRegExMatchAttribute(string dependentValue, string pattern) : base(dependentValue, Operator.NotRegExMatch, pattern) { }
    }
}
