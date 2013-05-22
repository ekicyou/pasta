namespace NeoDatis.Odb.Impl.Core.Query.Criteria
{
	/// <summary>A criterio to test collection or array size</summary>
	/// <author>olivier s</author>
	[System.Serializable]
	public class CollectionSizeCriterion : NeoDatis.Odb.Core.Query.Criteria.AbstractCriterion
	{
		public const int SizeEq = 1;

		public const int SizeNe = 2;

		public const int SizeGt = 3;

		public const int SizeGe = 4;

		public const int SizeLt = 5;

		public const int SizeLe = 6;

		private int size;

		private int sizeType;

		public CollectionSizeCriterion(string attributeName, int size, int sizeType) : base
			(attributeName)
		{
			// The size that the collection must have
			this.size = size;
			this.sizeType = sizeType;
		}

		public override bool Match(object valueToMatch)
		{
			// If it is a AttributeValuesMap, then gets the real value from the map
			if (valueToMatch is NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap)
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap attributeValues = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap
					)valueToMatch;
				valueToMatch = attributeValues.GetAttributeValue(attributeName);
			}
			if (valueToMatch == null)
			{
				// Null list are considered 0-sized list
				if (sizeType == SizeEq && size == 0)
				{
					return true;
				}
				if ((sizeType == SizeLe && size >= 0) || (sizeType == SizeLt && size > 0))
				{
					return true;
				}
				if (sizeType == SizeNe && size != 0)
				{
					return true;
				}
				return false;
			}
			if (valueToMatch is System.Collections.ICollection)
			{
				System.Collections.ICollection c = (System.Collections.ICollection)valueToMatch;
				return MatchSize(c.Count, size, sizeType);
			}
			System.Type clazz = valueToMatch.GetType();
			if (clazz.IsArray)
			{
				int arrayLength = NeoDatis.Tool.Wrappers.OdbReflection.GetArrayLength(valueToMatch
					);
				return MatchSize(arrayLength, size, sizeType);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryBadCriteria
				.AddParameter(valueToMatch.GetType().FullName));
		}

		private bool MatchSize(int collectionSize, int requestedSize, int sizeType)
		{
			switch (sizeType)
			{
				case SizeEq:
				{
					return collectionSize == requestedSize;
				}

				case SizeNe:
				{
					return collectionSize != requestedSize;
				}

				case SizeGt:
				{
					return collectionSize > requestedSize;
				}

				case SizeGe:
				{
					return collectionSize >= requestedSize;
				}

				case SizeLt:
				{
					return collectionSize < requestedSize;
				}

				case SizeLe:
				{
					return collectionSize <= requestedSize;
				}
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryCollectionSizeCriteriaNotSupported
				.AddParameter(sizeType));
		}

		public override NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap GetValues
			()
		{
			return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap();
		}

		public override void Ready()
		{
		}
	}
}
