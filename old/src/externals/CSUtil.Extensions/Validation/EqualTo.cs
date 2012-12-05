using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    public class EqualToAttribute : IsAttribute
    {
        public EqualToAttribute(string dependentProperty) : base(Operator.EqualTo, dependentProperty) { }
    }
}
