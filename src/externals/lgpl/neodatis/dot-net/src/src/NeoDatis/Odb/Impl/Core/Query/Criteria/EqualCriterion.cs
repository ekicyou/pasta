namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	/// <summary>A criterion to match equality</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class EqualCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		private object criterionValue;

		private bool isCaseSensitive;

		/// <summary>
		/// For criteria query on objects, we use the oid of the object instead of
		/// the object itself.
		/// </summary>
		/// <remarks>
		/// For criteria query on objects, we use the oid of the object instead of
		/// the object itself. So comparison will be done with OID It is faster and
		/// avoid the need of the object (class) having to implement Serializable in
		/// client server mode
		/// </remarks>
		private NeoDatis.Odb.OID oid;

		private bool objectIsNative;

		public EqualCriterion(string attributeName, int value) : base(attributeName)
		{
			Init(value);
		}

		public EqualCriterion(string attributeName, short value) : base(attributeName)
		{
			Init(value);
		}

		public EqualCriterion(string attributeName, byte value) : base(attributeName)
		{
			Init(value);
		}

		public EqualCriterion(string attributeName, float value) : base(attributeName)
		{
			Init(value);
		}

		public EqualCriterion(string attributeName, double value) : base(attributeName)
		{
			Init(value);
		}

		public EqualCriterion(string attributeName, long value) : base(attributeName)
		{
			Init(value);
		}

		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		public EqualCriterion(string attributeName, object value) : base(attributeName)
		{
			Init(value);
		}

		protected virtual void Init(object value)
		{
			this.criterionValue = value;
			this.isCaseSensitive = true;
			if (criterionValue == null)
			{
				this.objectIsNative = true;
			}
			else
			{
				this.objectIsNative = NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsNative(criterionValue
					.GetType());
			}
		}

		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		/// <param name="isCaseSensitive"></param>
		public EqualCriterion(string attributeName, object value, bool isCaseSensitive) : 
			base(attributeName)
		{
			this.criterionValue = value;
			this.isCaseSensitive = isCaseSensitive;
		}

		public EqualCriterion(string attributeName, string value, bool isCaseSensitive) : 
			base(attributeName)
		{
			this.criterionValue = value;
			this.isCaseSensitive = isCaseSensitive;
		}

		public EqualCriterion(string attributeName, bool value) : base(attributeName)
		{
			Init(value ? true : false);
		}

		public override bool Match(object valueToMatch)
		{
			// If it is a AttributeValuesMap, then gets the real value from the map
			// AttributeValuesMap is used to optimize Criteria Query
			// (reading only values of the object that the query needs to be
			// evaluated instead of reading the entire object)
			if (valueToMatch is NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap attributeValues = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
					)valueToMatch;
				valueToMatch = attributeValues.GetAttributeValue(attributeName);
			}
			if (valueToMatch == null && criterionValue == null && oid == null)
			{
				return true;
			}
			// if case sensitive (default value), just call the equals on the
			// objects
			if (isCaseSensitive)
			{
				if (objectIsNative)
				{
					return valueToMatch != null && NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare.AttributeValueComparator
						.Equals(valueToMatch, criterionValue);
				}
				NeoDatis.Odb.OID objectOid = (NeoDatis.Odb.OID)valueToMatch;
				if (oid == null)
				{
					// TODO Should we return false or thrown exception?
					// See junit TestCriteriaQuery6.test1
					return false;
				}
				// throw new
				// ODBRuntimeException(NeoDatisError.CRITERIA_QUERY_ON_UNKNOWN_OBJECT);
				return oid.Equals(objectOid);
			}
			// && valueToMatch.equals(criterionValue);
			// Case insensitive (iequal) only works on String or Character!
			bool canUseCaseInsensitive = (criterionValue.GetType() == typeof(string) && valueToMatch
				.GetType() == typeof(string)) || (criterionValue.GetType() == typeof(char) && valueToMatch
				.GetType() == typeof(char));
			if (!canUseCaseInsensitive)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryAttributeTypeNotSupportedInIequalExpression
					.AddParameter(valueToMatch.GetType().FullName));
			}
			// Cast to string to make the right comparison using the
			// equalsIgnoreCase
			string s1 = (string)valueToMatch;
			string s2 = (string)criterionValue;
			return NeoDatis.Tool.Wrappers.OdbString.EqualsIgnoreCase(s1, s2);
		}

		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(attributeName).Append(" = ").Append(criterionValue);
			return buffer.ToString();
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap map = new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
				();
			if (criterionValue == null && oid != null)
			{
				map.SetOid(oid);
			}
			else
			{
				map.Add(attributeName, criterionValue);
			}
			return map;
		}

		public override bool CanUseIndex()
		{
			return true;
		}

		public override void Ready()
		{
			if (!objectIsNative)
			{
				if (GetQuery() == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ContainsQueryWithNoQuery
						);
				}
				NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = GetQuery().GetStorageEngine
					();
				if (engine == null)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ContainsQueryWithNoStorageEngine
						);
				}
				// For non native object, we just need the oid of it
				oid = engine.GetObjectId(criterionValue, false);
				this.criterionValue = null;
			}
		}
	}
}
