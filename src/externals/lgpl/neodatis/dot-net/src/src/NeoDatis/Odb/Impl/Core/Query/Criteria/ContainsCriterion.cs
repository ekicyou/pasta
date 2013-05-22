namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	[System.Serializable]
	public class ContainsCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		private object criterionValue;

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

		public ContainsCriterion(string attributeName, string criterionValue) : base(attributeName
			)
		{
			Init(criterionValue);
		}

		public ContainsCriterion(string attributeName, int value) : base(attributeName)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, short value) : base(attributeName)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, byte value) : base(attributeName)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, float value) : base(attributeName)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, double value) : base(attributeName
			)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, long value) : base(attributeName)
		{
			Init(value);
		}

		protected virtual void Init(object value)
		{
			this.criterionValue = value;
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

		public ContainsCriterion(string attributeName, object value) : base(attributeName
			)
		{
			Init(value);
		}

		public ContainsCriterion(string attributeName, bool value) : base(attributeName)
		{
			Init(value ? true : false);
		}

		public override bool Match(object valueToMatch)
		{
			if (valueToMatch == null && criterionValue == null && oid == null)
			{
				return true;
			}
			if (valueToMatch == null)
			{
				return false;
			}
			System.Collections.IDictionary m = null;
			if (valueToMatch is System.Collections.IDictionary)
			{
				// The value in the map, just take the object with the attributeName
				m = (System.Collections.IDictionary)valueToMatch;
				valueToMatch = m[attributeName];
				// The value valueToMatch was redefined, so we need to re-make some
				// tests
				if (valueToMatch == null && criterionValue == null && oid == null)
				{
					return true;
				}
				if (valueToMatch == null)
				{
					return false;
				}
			}
			if (valueToMatch is System.Collections.ICollection)
			{
				System.Collections.ICollection c = (System.Collections.ICollection)valueToMatch;
				return CheckIfCollectionContainsValue(c);
			}
			System.Type clazz = valueToMatch.GetType();
			if (clazz.IsArray)
			{
				return CheckIfArrayContainsValue(valueToMatch);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryContainsCriterionTypeNotSupported
				.AddParameter(valueToMatch.GetType().FullName));
		}

		private bool CheckIfCollectionContainsValue(System.Collections.ICollection c)
		{
			NeoDatis.Odb.Core.Layers.Layer3.IStorageEngine engine = GetQuery().GetStorageEngine
				();
			if (engine == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryEngineNotSet
					);
			}
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			System.Collections.IEnumerator iterator = c.GetEnumerator();
			// If the object to compared is native
			if (objectIsNative)
			{
				while (iterator.MoveNext())
				{
					aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)iterator.Current;
					if (aoi == null && criterionValue == null)
					{
						return true;
					}
					if (aoi != null && criterionValue == null)
					{
						return false;
					}
					if (criterionValue.Equals(aoi.GetObject()))
					{
						return true;
					}
				}
				return false;
			}
			// Object is not native
			while (iterator.MoveNext())
			{
				aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)iterator.Current;
				if (aoi.IsNull() && criterionValue == null && oid == null)
				{
					return true;
				}
				if (aoi != null & oid != null)
				{
					if (aoi.IsNonNativeObject())
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi1 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
							)aoi;
						bool isEqual = nnoi1.GetOid() != null && oid != null && nnoi1.GetOid().Equals(oid
							);
						if (isEqual)
						{
							return true;
						}
					}
				}
			}
			return false;
		}

		private bool CheckIfArrayContainsValue(object valueToMatch)
		{
			int arrayLength = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayLength(valueToMatch
				);
			object element = null;
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			for (int i = 0; i < arrayLength; i++)
			{
				element = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayElement(valueToMatch, i);
				if (element == null && criterionValue == null)
				{
					return true;
				}
				aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)element;
				if (aoi != null && aoi.GetObject() != null && aoi.GetObject().Equals(criterionValue
					))
				{
					return true;
				}
			}
			return false;
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap();
		}

		public override void SetQuery(NeoDatis.Odb.Core.Query.IQuery query)
		{
			base.SetQuery(query);
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
