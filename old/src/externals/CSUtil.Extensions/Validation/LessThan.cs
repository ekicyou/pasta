using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    public class LessThanAttribute : IsAttribute
    {
        public LessThanAttribute(string dependentProperty) : base(Operator.LessThan, dependentProperty) { }
    }
}
