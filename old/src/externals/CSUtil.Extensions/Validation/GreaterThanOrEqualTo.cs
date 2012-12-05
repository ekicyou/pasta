using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    public class GreaterThanOrEqualToAttribute : IsAttribute
    {
        public GreaterThanOrEqualToAttribute(string dependentProperty) : base(Operator.GreaterThanOrEqualTo, dependentProperty) { }
    }
}
