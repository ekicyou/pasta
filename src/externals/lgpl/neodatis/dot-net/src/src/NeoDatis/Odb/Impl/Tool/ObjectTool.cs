namespace NeoDatis.Odb.Impl.Tool
{
	/// <summary>Basic native Object formatter.</summary>
	/// <remarks>Basic native Object formatter. Used in ODBExplorer and XML import/export.
	/// 	</remarks>
	/// <author>osmadja</author>
	public class ObjectTool
	{
		public static NeoDatis.Tool.Wrappers.OdbDateFormat format = new NeoDatis.Tool.Wrappers.OdbDateFormat
			("dd/MM/yyyy HH:mm:ss:SSS");

		public static int IdCallerIsOdbExplorer = 1;

		public static int IdCallerIsXml = 2;

		public static int IdCallerIsSerializer = 2;

		/// <summary>
		/// Convert a string representation to the right object
		/// <pre>
		/// If it is a representation of an int, it will return an Integer.
		/// </summary>
		/// <remarks>
		/// Convert a string representation to the right object
		/// <pre>
		/// If it is a representation of an int, it will return an Integer.
		/// </pre>
		/// </remarks>
		/// <param name="odbTypeId">The native object type</param>
		/// <param name="value">The real value</param>
		/// <param name="caller">
		/// The caller type , can be one of the constants
		/// ObjectTool.CALLER_IS_
		/// </param>
		/// <returns>The right object</returns>
		/// <exception cref="Java.Lang.NumberFormatException">Java.Lang.NumberFormatException
		/// 	</exception>
		/// <exception cref="Java.Text.ParseException">Java.Text.ParseException</exception>
		public static object StringToObject(int odbTypeId, string value, int caller)
		{
			object theObject = null;
			if (value == null || value.Equals("null"))
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo(odbTypeId);
			}
			switch (odbTypeId)
			{
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeByteId:
				{
					theObject = System.Convert.ToByte(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeBooleanId:
				{
					theObject = value.Equals("true") ? true : false;
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeCharId:
				{
					theObject = value[0];
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeFloatId:
				{
					theObject = System.Convert.ToSingle(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeDoubleId:
				{
					theObject = System.Convert.ToDouble(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeIntId:
				{
					theObject = System.Convert.ToInt32(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeLongId:
				{
					theObject = System.Convert.ToInt64(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.NativeShortId:
				{
					theObject = System.Convert.ToInt16(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigDecimalId:
				{
					theObject = NeoDatis.Tool.Wrappers.NeoDatisNumber.CreateDecimalFromString(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BigIntegerId:
				{
					theObject = NeoDatis.Tool.Wrappers.NeoDatisNumber.CreateBigIntegerFromString(value
						);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.BooleanId:
				{
					theObject = value.Equals("true") ? true : false;
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.CharacterId:
				{
					theObject = value[0];
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateSqlId:
				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateTimestampId:
				{
					if (NeoDatis.Odb.Impl.Tool.ObjectTool.CallerIsOdbExplorer(caller))
					{
						theObject = format.Parse(value);
					}
					if (NeoDatis.Odb.Impl.Tool.ObjectTool.CallerIsXml(caller) || NeoDatis.Odb.Impl.Tool.ObjectTool
						.CallerIsSerializer(caller))
					{
						theObject = new System.DateTime(long.Parse(value));
					}
					System.DateTime date = (System.DateTime)theObject;
					if (odbTypeId == NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateSqlId)
					{
						theObject = new System.DateTime(date.Millisecond);
					}
					if (odbTypeId == NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DateTimestampId)
					{
						theObject = new System.DateTime(date.Millisecond);
					}
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.FloatId:
				{
					theObject = System.Convert.ToSingle(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.DoubleId:
				{
					theObject = System.Convert.ToDouble(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IntegerId:
				{
					theObject = System.Convert.ToInt32(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.LongId:
				{
					theObject = System.Convert.ToInt64(value);
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.StringId:
				{
					theObject = value;
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.OidId:
				{
					theObject = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(long.Parse(value));
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ObjectOidId:
				{
					theObject = NeoDatis.Odb.Core.Oid.OIDFactory.BuildObjectOID(long.Parse(value));
					break;
				}

				case NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.ClassOidId:
				{
					theObject = NeoDatis.Odb.Core.Oid.OIDFactory.BuildClassOID(long.Parse(value));
					break;
				}
			}
			if (theObject == null)
			{
				throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.NativeTypeNotSupported
					.AddParameter(NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.GetNameFromId(odbTypeId
					)));
			}
			return theObject;
		}

		/// <param name="odbTypeId">The native object type</param>
		/// <param name="value">The real value</param>
		/// <param name="caller">
		/// The caller type , can be one of the constants
		/// ObjectTool.CALLER_IS_
		/// </param>
		/// <param name="ci">
		/// The ClassInfo. It is only used for enum where we need the enum
		/// class info. In other cases, it is null
		/// </param>
		/// <returns>The NativeObjectInfo that represents the specific value</returns>
		/// <exception cref="Java.Lang.NumberFormatException">Java.Lang.NumberFormatException
		/// 	</exception>
		/// <exception cref="Java.Text.ParseException">Java.Text.ParseException</exception>
		public static NeoDatis.Odb.Core.Layers.Layer2.Meta.NativeObjectInfo StringToObjectInfo
			(int odbTypeId, string value, int caller, NeoDatis.Odb.Core.Layers.Layer2.Meta.ClassInfo
			 ci)
		{
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsAtomicNative(odbTypeId))
			{
				object theObject = StringToObject(odbTypeId, value, caller);
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo(theObject, 
					odbTypeId);
			}
			if (NeoDatis.Odb.Core.Layers.Layer2.Meta.ODBType.IsEnum(odbTypeId))
			{
				return new NeoDatis.Odb.Core.Layers.Layer2.Meta.EnumNativeObjectInfo(ci, value);
			}
			return NeoDatis.Odb.Core.Layers.Layer2.Meta.NullNativeObjectInfo.GetInstance();
		}

		public static string AtomicNativeObjectToString(NeoDatis.Odb.Core.Layers.Layer2.Meta.AtomicNativeObjectInfo
			 anoi, int caller)
		{
			if (anoi == null || anoi.IsNull())
			{
				return "null";
			}
			if (anoi.GetObject() is System.DateTime)
			{
				if (NeoDatis.Odb.Impl.Tool.ObjectTool.CallerIsOdbExplorer(caller))
				{
					return format.Format((System.DateTime)anoi.GetObject());
				}
				return ((System.DateTime)anoi.GetObject()).Millisecond.ToString();
			}
			return anoi.GetObject().ToString();
		}

		public static bool CallerIsOdbExplorer(int caller)
		{
			return caller == IdCallerIsOdbExplorer;
		}

		public static bool CallerIsXml(int caller)
		{
			return caller == IdCallerIsXml;
		}

		public static bool CallerIsSerializer(int caller)
		{
			return caller == IdCallerIsSerializer;
		}
	}
}
