namespace NeoDatis.Odb.Core.Query.Execution
{
	public class IndexTool
	{
		public static NeoDatis.Tool.Wrappers.OdbComparable BuildIndexKey(string indexName
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo oi, int[] fieldIds)
		{
			NeoDatis.Tool.Wrappers.OdbComparable[] keys = new NeoDatis.Tool.Wrappers.OdbComparable
				[fieldIds.Length];
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo aoi = null;
			System.IComparable o = null;
			for (int i = 0; i < fieldIds.Length; i++)
			{
				// Todo : can we assume that the object is a Comparable
				try
				{
					aoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo)oi.GetAttributeValueFromId
						(fieldIds[i]);
					o = (System.IComparable)aoi.GetObject();
					// JDK1.4 restriction: Boolean is not Comparable in jdk1.4
					if (aoi.GetOdbType().IsBoolean())
					{
						bool b = (bool)o;
						if (b)
						{
							o = (byte)1;
						}
						else
						{
							o = (byte)0;
						}
					}
					// If the index is on NonNativeObjectInfo, then the key is the oid 
					// of the object
					if (aoi.IsNonNativeObject())
					{
						NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
							)aoi;
						o = nnoi.GetOid();
					}
					keys[i] = new NeoDatis.Odb.Core.Query.SimpleCompareKey(o);
				}
				catch (System.Exception)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexKeysMustImplementComparable
						.AddParameter(fieldIds[i]).AddParameter(oi.GetAttributeValueFromId(fieldIds[i]).
						GetType().FullName));
				}
			}
			if (keys.Length == 1)
			{
				return keys[0];
			}
			return new NeoDatis.Odb.Core.Query.ComposedCompareKey(keys);
		}

		public static NeoDatis.Tool.Wrappers.OdbComparable BuildIndexKey(string indexName
			, NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values, string[] fields
			)
		{
			if (fields.Length == 1)
			{
				return new NeoDatis.Odb.Core.Query.SimpleCompareKey(values.GetComparable(fields[0
					]));
			}
			NeoDatis.Tool.Wrappers.OdbComparable[] keys = new NeoDatis.Tool.Wrappers.OdbComparable
				[fields.Length];
			System.IComparable @object = null;
			for (int i = 0; i < fields.Length; i++)
			{
				// Todo : can we assume that the object is a Comparable
				try
				{
					@object = (System.IComparable)values[fields[i]];
					// JDK1.4 restriction: Boolean is not Comparable in jdk1.4
					if (@object is bool)
					{
						bool b = (bool)@object;
						if (b)
						{
							@object = (byte)1;
						}
						else
						{
							@object = (byte)0;
						}
					}
					keys[i] = new NeoDatis.Odb.Core.Query.SimpleCompareKey(@object);
				}
				catch (System.Exception)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.IndexKeysMustImplementComparable
						.AddParameter(indexName).AddParameter(fields[i]).AddParameter(values[fields[i]].
						GetType().FullName));
				}
			}
			NeoDatis.Odb.Core.Query.ComposedCompareKey key = new NeoDatis.Odb.Core.Query.ComposedCompareKey
				(keys);
			return key;
		}

		/// <summary>Take the fields of the index and take value from the query</summary>
		/// <param name="ci">The class info involved</param>
		/// <param name="index">The index</param>
		/// <param name="query"></param>
		/// <returns>The key of the index</returns>
		public static NeoDatis.Tool.Wrappers.OdbComparable ComputeKey(NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfoIndex index, NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
			 query)
		{
			string[] attributesNames = ci.GetAttributeNames(index.GetAttributeIds());
			NeoDatis.Odb.Core.Layers.Layer2.Meta.AttributeValuesMap values = query.GetCriteria
				().GetValues();
			return BuildIndexKey(index.GetName(), values, attributesNames);
		}
	}
}
