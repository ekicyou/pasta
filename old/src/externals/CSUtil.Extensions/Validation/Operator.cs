using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    /// <summary>Operator</summary>
    public enum Operator
    {
        /// <summary>EqualTo</summary>
        EqualTo,
        /// <summary>NotEqualTo</summary>
        NotEqualTo,
        /// <summary>GreaterThan</summary>
        GreaterThan,
        /// <summary>LessThan</summary>
        LessThan,
        /// <summary>GreaterThanOrEqualTo</summary>
        GreaterThanOrEqualTo,
        /// <summary>LessThanOrEqualTo</summary>
        LessThanOrEqualTo,
        /// <summary>RegExMatch</summary>
        RegExMatch,
        /// <summary>NotRegExMatch</summary>
        NotRegExMatch
    }
}
