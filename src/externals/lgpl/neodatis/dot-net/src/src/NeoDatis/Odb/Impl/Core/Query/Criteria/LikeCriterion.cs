using NeoDatis.Tool.Wrappers;
using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core;
namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	[System.Serializable]
	public class LikeCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		private string criterionValue;

		private bool isCaseSensitive;

		public LikeCriterion(string attributeName, string criterionValue, bool isCaseSensiive
			) : base(attributeName)
		{
			this.criterionValue = criterionValue;
			this.isCaseSensitive = isCaseSensiive;
		}

		public override bool Match(object valueToMatch)
		{
			string regExp = null;
			if (valueToMatch == null)
			{
				return false;
			}
			// If it is a AttributeValuesMap, then gets the real value from the map
			if (valueToMatch is AttributeValuesMap)
			{
				AttributeValuesMap attributeValues = (AttributeValuesMap)valueToMatch;
				valueToMatch = attributeValues[attributeName];
			}
			if (valueToMatch == null)
			{
				return false;
			}
			// Like operator only work with String
			if (!(valueToMatch is string))
			{
				throw new ODBRuntimeException(NeoDatisError.QueryAttributeTypeNotSupportedInLikeExpression
					.AddParameter(valueToMatch.GetType().FullName));
			}
			string value = (string)valueToMatch;
			if (criterionValue.IndexOf("%") != -1)
			{
				regExp = OdbString.ReplaceToken(criterionValue, "%", "(.)*");
				if (isCaseSensitive)
				{
                    bool b = value != null && OdbString.Matches(regExp, value);
                    return b;
				}
				return value != null && OdbString.Matches(regExp.ToLower()
					, value.ToLower());
			}
			if (isCaseSensitive)
			{
				regExp = string.Format("(.)*%s(.)*", criterionValue);
				return value != null && OdbString.Matches(regExp, value);
			}
			regExp = string.Format("(.)*%s(.)*", criterionValue.ToLower());
			return value != null && OdbString.Matches(regExp, value.ToLower());
		}

		public override AttributeValuesMap GetValues()
		{
			return new AttributeValuesMap();
		}

		public override void Ready()
		{
		}
	}
}
