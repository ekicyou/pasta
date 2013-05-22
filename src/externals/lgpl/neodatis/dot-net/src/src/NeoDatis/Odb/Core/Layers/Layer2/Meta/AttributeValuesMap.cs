using System.Collections;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>A Map to contain values of attributes of an object.</summary>
	/// <remarks>
	/// A Map to contain values of attributes of an object.
	/// It is used to optimize a criteria query execution where ODB , while reading an instance data, tries to retrieve only values
	/// of attributes involved in the query instead of reading the entire object.
	/// </remarks>
	/// <author>olivier s</author>
	[System.Serializable]
    public class AttributeValuesMap : System.Collections.Hashtable
		
	{
		/// <summary>The Object Info Header of the object being represented</summary>
		private NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader objectInfoHeader;

		/// <summary>The oid of the object.</summary>
		/// <remarks>
		/// The oid of the object. This is used when some criteria (example is equalCriterion) is on an object,
		/// in this case the comparison is done on the oid of the object and not on the object itself.
		/// </remarks>
		private NeoDatis.Odb.OID oid;

		public virtual NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader GetObjectInfoHeader
			()
		{
			return objectInfoHeader;
		}

		public virtual void SetObjectInfoHeader(NeoDatis.Odb.Core.Layers.Layer2.Meta.ObjectInfoHeader
			 objectInfoHeader)
		{
			this.objectInfoHeader = objectInfoHeader;
		}

		public virtual object GetAttributeValue(string attributeName)
		{
			return this[attributeName];
		}

		public virtual System.IComparable GetComparable(string attributeName)
		{
			return (System.IComparable)GetAttributeValue(attributeName);
		}

		public virtual bool HasOid()
		{
			return oid != null;
		}

		public virtual NeoDatis.Odb.OID GetOid()
		{
			return oid;
		}

		public virtual void SetOid(NeoDatis.Odb.OID oid)
		{
			this.oid = oid;
		}

        public bool PutAll(IDictionary map)
        {

            ICollection keys = map.Keys;
            foreach (object k in keys)
            {
                Add(k, map[k]);
            }
            return true;
        }
	}
}
