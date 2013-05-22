namespace NeoDatis.Odb.Core.Query.Criteria
{
	/// <summary>A Criterion for greater than (gt),greater or equal(ge), less than (lt) and less or equal (le)
	/// 	</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class ComparisonCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		public const int ComparisonTypeGt = 1;

		public const int ComparisonTypeGe = 2;

		public const int ComparisonTypeLt = 3;

		public const int ComparisonTypeLe = 4;

		private object criterionValue;

		private int comparisonType;

		public ComparisonCriterion(string attributeName, string criterionValue, int comparisonType
			) : base(attributeName)
		{
			this.criterionValue = criterionValue;
		}

		public ComparisonCriterion(string attributeName, int value, int comparisonType) : 
			base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, short value, int comparisonType)
			 : base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, byte value, int comparisonType) : 
			base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, float value, int comparisonType)
			 : base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, double value, int comparisonType
			) : base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, long value, int comparisonType) : 
			base(attributeName)
		{
			Init(value, comparisonType);
		}

		public ComparisonCriterion(string attributeName, object value, int comparisonType
			) : base(attributeName)
		{
			Init(value, comparisonType);
		}

		protected virtual void Init(object value, int comparisonType)
		{
			this.criterionValue = value;
			this.comparisonType = comparisonType;
		}

		public ComparisonCriterion(string attributeName, bool value, int comparisonType) : 
			base(attributeName)
		{
			Init(value ? true : false, comparisonType);
		}

		public override bool Match(object valueToMatch)
		{
			if (valueToMatch == null && criterionValue == null)
			{
				return true;
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap attributeValues = null;
			// If it is a AttributeValuesMap, then gets the real value from the map 
			if (valueToMatch is NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)
			{
				attributeValues = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)valueToMatch;
				valueToMatch = attributeValues.GetAttributeValue(attributeName);
			}
			if (valueToMatch == null)
			{
				return false;
			}
			if (!(valueToMatch is System.IComparable))
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryComparableCriteriaAppliedOnNonComparable
					.AddParameter(valueToMatch.GetType().FullName));
			}
			System.IComparable comparable1 = (System.IComparable)valueToMatch;
			System.IComparable comparable2 = (System.IComparable)criterionValue;
			switch (comparisonType)
			{
				case ComparisonTypeGt:
				{
					return valueToMatch != null && NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
						.Compare(comparable1, comparable2) > 0;
				}

				case ComparisonTypeGe:
				{
					return valueToMatch != null && NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
						.Compare(comparable1, comparable2) >= 0;
				}

				case ComparisonTypeLt:
				{
					return valueToMatch != null && NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
						.Compare(comparable1, comparable2) < 0;
				}

				case ComparisonTypeLe:
				{
					return valueToMatch != null && NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
						.Compare(comparable1, comparable2) <= 0;
				}
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryUnknownOperator
				.AddParameter(comparisonType));
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(attributeName).Append(" ").Append(GetOperator()).Append(" ").Append
				(criterionValue);
			return buffer.ToString();
		}

		private string GetOperator()
		{
			switch (comparisonType)
			{
				case ComparisonTypeGt:
				{
					return ">";
				}

				case ComparisonTypeGe:
				{
					return ">=";
				}

				case ComparisonTypeLt:
				{
					return "<";
				}

				case ComparisonTypeLe:
				{
					return "<=";
				}
			}
			return "?";
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap map = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
				();
			map.Add(attributeName, criterionValue);
			return map;
		}

		public override void Ready()
		{
		}
	}
}
