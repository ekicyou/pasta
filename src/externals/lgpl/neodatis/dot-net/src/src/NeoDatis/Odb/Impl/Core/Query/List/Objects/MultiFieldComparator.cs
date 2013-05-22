namespace NeoDatis.Odb.Impl.Core.Query.List.Objects
{
	/// <summary>For list sorting.</summary>
	/// <remarks>For list sorting. Used by the Query.orderBy functionality.</remarks>
	/// <author>osmadja</author>
	public class MultiFieldComparator : System.Collections.IComparer
	{
		private string[] fieldNames;

		private System.Collections.IDictionary map;

		private int way;

		private NeoDatis.Odb.Core.Layers.Layer1.Introspector.IClassIntrospector classIntrospector;

		public MultiFieldComparator(string[] names, NeoDatis.Odb.Core.OrderByConstants orderByType
			)
		{
			this.fieldNames = names;
			map = new System.Collections.Hashtable();
			this.way = orderByType.IsOrderByAsc() ? 1 : -1;
			this.classIntrospector = NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector
				();
		}

		public virtual int Compare(object o1, object o2)
		{
			int fieldIndex = 0;
			int result = CompareField(fieldIndex, o1, o2);
			fieldIndex++;
			while (result == 0 && fieldIndex < fieldNames.Length)
			{
				result = CompareField(fieldIndex, o1, o2);
				fieldIndex++;
			}
			return result;
		}

		public virtual int CompareField(int fieldIndex, object o1, object o2)
		{
			//Field field2 = getField(o2);
			object oo1 = null;
			object oo2 = null;
			System.Type type = null;
			try
			{
				if (o1 is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo && o2 is NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo)
				{
					NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi1 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
						)o1;
					NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo nnoi2 = (NeoDatis.Odb.Core.Layers.Layer2.Meta.NonNativeObjectInfo
						)o2;
					oo1 = nnoi1.GetValueOf(fieldNames[fieldIndex]);
					oo2 = nnoi2.GetValueOf(fieldNames[fieldIndex]);
					type = oo1.GetType();
				}
				else
				{
					System.Reflection.FieldInfo field1 = GetField(fieldIndex, o1);
					oo1 = field1.GetValue(o1);
					oo2 = field1.GetValue(o2);
					type = field1.GetType();
				}
				// if(isNumeric(field1) && isNumeric(field2)){
				if (type == typeof(int))
				{
					return way * CompareTo((int)oo1, (int)oo2);
				}
				if (type == typeof(long))
				{
					return way * CompareTo((long)oo1, (long)oo2);
				}
				if (type == typeof(short))
				{
					return way * CompareTo((short)oo1, (short)oo2);
				}
				if (type == typeof(double))
				{
					return way * CompareTo((double)oo1, (double)oo2);
				}
				if (type == typeof(float))
				{
					return way * CompareTo((float)oo1, (float)oo2);
				}
				if (type == typeof(byte))
				{
					return way * CompareTo((byte)oo1, (byte)oo2);
				}
				if (type == typeof(int))
				{
					return way * CompareTo((int)oo1, (int)oo2);
				}
				if (type == typeof(long))
				{
					return way * CompareToLong((long)oo1, (long)oo2);
				}
				if (type == typeof(System.DateTime))
				{
					return way * CompareTo((System.DateTime)oo1, (System.DateTime)oo2);
				}
				return way * CompareTo(oo1, oo2);
			}
			catch (System.Exception e)
			{
				throw new System.Exception(e.Message);
			}
		}

		private int CompareTo(object o1, object o2)
		{
			return o1.ToString().CompareTo(o2.ToString());
		}

		private int CompareTo(System.DateTime o1, System.DateTime o2)
		{
			return o1.CompareTo(o2);
		}

		private int CompareTo(long o1, long o2)
		{
			return (int)(o1 - o2);
		}

		private int CompareTo(byte o1, byte o2)
		{
			return o1.CompareTo(o2);
		}

		private int CompareTo(short o1, short o2)
		{
			return o1.CompareTo(o2);
		}

		private int CompareTo(float o1, float o2)
		{
			return o1.CompareTo(o2);
		}

		private int CompareToLong(long o1, long o2)
		{
			return o1.CompareTo(o2);
		}

		private int CompareTo(double o1, double o2)
		{
			return o1.CompareTo(o2);
		}

		private bool IsNumeric(System.Reflection.FieldInfo field)
		{
			System.Type clazz = field.GetType();
			bool b = clazz == typeof(int) || clazz == typeof(long);
			if (b)
			{
				return b;
			}
			b = clazz == typeof(long) || clazz == typeof(int);
			if (b)
			{
				return b;
			}
			b = clazz == typeof(float) || clazz == typeof(double);
			if (b)
			{
				return b;
			}
			b = clazz == typeof(System.Decimal) || clazz == typeof(System.Decimal);
			if (b)
			{
				return b;
			}
			b = clazz == typeof(short) || clazz == typeof(byte);
			return b;
		}

		private System.Reflection.FieldInfo GetField(int fieldIndex, object o)
		{
			System.Type clazz = o.GetType();
			string key = clazz.FullName + fieldNames[fieldIndex];
			System.Reflection.FieldInfo field = (System.Reflection.FieldInfo)map[key];
			if (field != null)
			{
				return field;
			}
			System.Collections.IList l = classIntrospector.GetAllFields(clazz.FullName);
			for (int i = 0; i < l.Count; i++)
			{
				field = (System.Reflection.FieldInfo)l[i];
				if (field.Name.Equals(fieldNames[fieldIndex]))
				{
					map.Add(key, field);
					return field;
				}
			}
			throw new System.Exception("Field " + fieldNames[fieldIndex] + " does not exist on class "
				 + o.GetType());
		}
	}
}
