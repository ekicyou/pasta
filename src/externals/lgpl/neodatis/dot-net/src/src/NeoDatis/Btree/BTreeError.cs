namespace NeoDatis.Btree
{
	/// <summary>ODB BTree Errors All @ in error description will be replaced by parameters
	/// 	</summary>
	/// <author>olivier s</author>
	public class BTreeError : NeoDatis.Odb.Core.IError
	{
		private int code;

		private string description;

		private System.Collections.IList parameters;

		public static readonly NeoDatis.Btree.BTreeError MergeWithTwoMoreKeys = new NeoDatis.Btree.BTreeError
			(500, "Trying to merge two node with more keys than allowed! @1 // @2");

		public static readonly NeoDatis.Btree.BTreeError LazyLoadingNode = new NeoDatis.Btree.BTreeError
			(501, "Error while loading node lazily with oid @1");

		public static readonly NeoDatis.Btree.BTreeError NodeWithoutId = new NeoDatis.Btree.BTreeError
			(502, "Node with id -1");

		public static readonly NeoDatis.Btree.BTreeError NullPersisterFound = new NeoDatis.Btree.BTreeError
			(503, "Null persister for PersistentBTree");

		public static readonly NeoDatis.Btree.BTreeError InvalidIdForBtree = new NeoDatis.Btree.BTreeError
			(504, "Invalid id for Btree : id=@1");

		public static readonly NeoDatis.Btree.BTreeError InvalidNodeType = new NeoDatis.Btree.BTreeError
			(505, "Node should be a PersistentNode but is a @1");

		public static readonly NeoDatis.Btree.BTreeError InternalError = new NeoDatis.Btree.BTreeError
			(506, "Internal error: @1");

		public BTreeError(int code, string description)
		{
			this.code = code;
			this.description = description;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(object o)
		{
			if (parameters == null)
			{
				parameters = new System.Collections.ArrayList();
			}
			parameters.Add(o != null ? o.ToString() : "null");
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(string s)
		{
			if (parameters == null)
			{
				parameters = new System.Collections.ArrayList();
			}
			parameters.Add(s);
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(int i)
		{
			if (parameters == null)
			{
				parameters = new System.Collections.ArrayList();
			}
			parameters.Add(i);
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(byte i)
		{
			if (parameters == null)
			{
				parameters = new System.Collections.ArrayList();
			}
			parameters.Add(i);
			return this;
		}

		public virtual NeoDatis.Odb.Core.IError AddParameter(long l)
		{
			if (parameters == null)
			{
				parameters = new System.Collections.ArrayList();
			}
			parameters.Add(l);
			return this;
		}

		/// <summary>replace the @1,@2,...</summary>
		/// <remarks>replace the @1,@2,... by their real values.</remarks>
		public override string ToString()
		{
			System.Text.StringBuilder buffer = new System.Text.StringBuilder();
			buffer.Append(code).Append(":").Append(description);
			string s = buffer.ToString();
			if (parameters != null)
			{
				for (int i = 0; i < parameters.Count; i++)
				{
					string parameterName = "@" + (i + 1);
					string parameterValue = parameters[i].ToString();
					int parameterIndex = s.IndexOf(parameterName);
					if (parameterIndex != -1)
					{
						s = NeoDatis.Tool.Wrappers.OdbString.ReplaceToken(s, parameterName, parameterValue
							, 1);
					}
				}
			}
			return s;
		}
	}
}
