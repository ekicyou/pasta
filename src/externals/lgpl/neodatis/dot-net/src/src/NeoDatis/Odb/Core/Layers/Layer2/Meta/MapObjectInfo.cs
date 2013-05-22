using System.Collections.Generic;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Core.Layers.Layer2.Meta
{
	/// <summary>Meta representation of a Map</summary>
	/// <author>osmadja</author>
	[System.Serializable]
	public class MapObjectInfo : NeoDatis.Odb.Core.Layers.Layer2.Meta.GroupObjectInfo
	{
		private string realMapClassName;

		public MapObjectInfo(IDictionary<AbstractObjectInfo,AbstractObjectInfo> map, string realMapClassName)
			 : base(map, NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.MapId)
		{
			this.realMapClassName = realMapClassName;
		}

		public MapObjectInfo(IDictionary<AbstractObjectInfo,AbstractObjectInfo> map, ODBType type, string realMapClassName) : base(map, type)
		{
			this.realMapClassName = realMapClassName;
		}

		public virtual System.Collections.Generic.IDictionary<AbstractObjectInfo,AbstractObjectInfo> GetMap()
		{
			return (IDictionary<AbstractObjectInfo,AbstractObjectInfo>)theObject;
		}

		public override string ToString()
		{
			if (theObject != null)
			{
				return theObject.ToString();
			}
			return "null map";
		}

		public override bool IsMapObject()
		{
			return true;
		}

		public virtual string GetRealMapClassName()
		{
			return realMapClassName;
		}

		public virtual void SetRealMapClassName(string realMapClassName)
		{
			this.realMapClassName = realMapClassName;
		}

		public override AbstractObjectInfo CreateCopy(IDictionary<OID, AbstractObjectInfo> cache, bool onlyData)
		{
            IDictionary<AbstractObjectInfo, AbstractObjectInfo> m = GetMap();
			IDictionary<AbstractObjectInfo,AbstractObjectInfo> newMap = new OdbHashMap<AbstractObjectInfo, AbstractObjectInfo>();
			System.Collections.IEnumerator iterator = m.Keys.GetEnumerator();
			while (iterator.MoveNext())
			{
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo keyAoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					)iterator.Current;
				NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo valueAoi = (NeoDatis.Odb.Core.Layers.Layer2.Meta.AbstractObjectInfo
					)m[keyAoi];
				// create copies
				keyAoi = keyAoi.CreateCopy(cache, onlyData);
				valueAoi = valueAoi.CreateCopy(cache, onlyData);
				newMap.Add(keyAoi, valueAoi);
			}
			MapObjectInfo moi = new MapObjectInfo(newMap, odbType, realMapClassName);
			return moi;
		}
	}
}
