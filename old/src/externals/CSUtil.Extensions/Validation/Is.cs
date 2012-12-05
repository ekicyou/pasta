using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace CSUtil.Validation
{
    public class IsAttribute : ContingentValidationAttribute
    {
        public Operator Operator { get; private set; }
        private OperatorMetadata _metadata;

        public IsAttribute(Operator @operator, string dependentProperty)
            : base(dependentProperty)
        {
            Operator = @operator;
            _metadata = OperatorMetadata.Get(Operator);
        }

        public override string ClientTypeName
        {
            get { return "Is"; }
        }

        protected override IEnumerable<KeyValuePair<string, object>> GetClientValidationParameters()
        {
            return base.GetClientValidationParameters()
                .Union(new [] { new KeyValuePair<string, object>("Operator", Operator.ToString()) });
        }

        public override bool IsValid(object value, object dependentValue, object container)
        {
            return _metadata.IsValid(value, dependentValue);
        }

        public override string DefaultErrorMessage
        {
            get { return "{0} must be " + _metadata.ErrorMessage + " {1}."; }
        }
    }
}
