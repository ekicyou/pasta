using NeoDatis.Tool.Wrappers;
namespace NeoDatis.Odb.Impl.Core.Query.Values
{
	/// <summary>A values Criteria quwry is a query to retrieve object values instead of objects.
	/// 	</summary>
	/// <remarks>
	/// A values Criteria quwry is a query to retrieve object values instead of objects. Values Criteria Query allows one to retrieve one field value of an object:
	/// - A field values
	/// - The sum of a specific numeric field
	/// - The Max value of a specific numeric field
	/// - The Min value of a specific numeric field
	/// - The Average value of a specific numeric value
	/// </remarks>
	/// <author>osmadja</author>
	[System.Serializable]
	public class ValuesCriteriaQuery : NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery
		, NeoDatis.Odb.Core.Query.IValuesQuery
	{
		private NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
			> objectActions;

		private string[] groupByFieldList;

		private bool hasGroupBy;

		/// <summary>To specify if the result must build instance of object meta representation
		/// 	</summary>
		private bool returnInstance;

        public ValuesCriteriaQuery(System.Type aClass, NeoDatis.Odb.OID oid) : base(OdbClassUtil.GetFullName(aClass))
		{
			SetOidOfObjectToQuery(oid);
			Init();
		}

		public ValuesCriteriaQuery(System.Type aClass, NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criteria) : base(aClass.FullName, criteria)
		{
			Init();
		}

		public ValuesCriteriaQuery(System.Type aClass) : base(OdbClassUtil.GetFullName(aClass))
		{
			Init();
		}

        public ValuesCriteriaQuery(string aFullClassName)  : base(aFullClassName)
		{
			Init();
		}

		public ValuesCriteriaQuery(string aFullClassName, NeoDatis.Odb.Core.Query.Criteria.ICriterion
             criteria) : base(aFullClassName, criteria)
		{
			Init();
		}

		public ValuesCriteriaQuery(NeoDatis.Odb.Impl.Core.Query.Criteria.CriteriaQuery query
			) : this(query.GetFullClassName(), query.GetCriteria())
		{
		}

		private void Init()
		{
			objectActions = new NeoDatis.Tool.Wrappers.List.OdbArrayList<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
				>();
			returnInstance = true;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Count(string alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.CountAction(alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sum(string attributeName)
		{
			return Sum(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sum(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.SumAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, 
			int fromIndex, int size, bool throwException)
		{
			return Sublist(attributeName, attributeName, fromIndex, size, throwException);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, 
			string alias, int fromIndex, int size, bool throwException)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.SublistAction(attributeName
				, alias, fromIndex, size, throwException));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, 
			int fromIndex, int toIndex)
		{
			return Sublist(attributeName, attributeName, fromIndex, toIndex);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Sublist(string attributeName, 
			string alias, int fromIndex, int toIndex)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.SublistAction(attributeName
				, alias, fromIndex, toIndex));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Size(string attributeName)
		{
			return Size(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Size(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.SizeAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Avg(string attributeName)
		{
			return Avg(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Avg(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.AverageValueAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Max(string attributeName)
		{
			return Max(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Max(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.MaxValueAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Min(string attributeName)
		{
			return Min(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Min(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.MinValueAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Field(string attributeName)
		{
			return Field(attributeName, attributeName);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Field(string attributeName, string
			 alias)
		{
			objectActions.Add(new NeoDatis.Odb.Impl.Core.Query.Values.FieldValueAction(attributeName
				, alias));
			return this;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Custom(string attributeName, 
			NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction action)
		{
			return Custom(attributeName, attributeName, action);
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery Custom(string attributeName, 
			string alias, NeoDatis.Odb.Core.Query.Values.ICustomQueryFieldAction action)
		{
			action.SetAttributeName(attributeName);
			action.SetAlias(alias);
			objectActions.Add(action);
			return this;
		}

		public virtual NeoDatis.Tool.Wrappers.List.IOdbList<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
			> GetObjectActions()
		{
			return objectActions;
		}

		/// <summary>Returns the list of involved fields for this query.</summary>
		/// <remarks>
		/// Returns the list of involved fields for this query. List of String
		/// <pre>
		/// If query must return sum("value") and field("name"), involvedField will contain "value","name"
		/// </pre>
		/// </remarks>
		public override NeoDatis.Tool.Wrappers.List.IOdbList<string> GetAllInvolvedFields
			()
		{
			NeoDatis.Tool.Wrappers.List.IOdbList<string> l = new NeoDatis.Tool.Wrappers.List.OdbArrayList
				<string>();
			// To check field duplicity
			System.Collections.Generic.IDictionary<string, string> map = new NeoDatis.Tool.Wrappers.Map.OdbHashMap
				<string, string>();
			l.AddAll(base.GetAllInvolvedFields());
			if (!l.IsEmpty())
			{
				for (int i = 0; i < l.Count; i++)
				{
					map.Add(l[i], l[i]);
				}
			}
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
				> iterator = objectActions.GetEnumerator();
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction oa = null;
			string name = null;
			while (iterator.MoveNext())
			{
				oa = iterator.Current;
				if (oa.GetType() != typeof(NeoDatis.Odb.Impl.Core.Query.Values.CountAction))
				{
					name = oa.GetAttributeName();
					if (!map.ContainsKey(name))
					{
						l.Add(name);
						map.Add(name, name);
					}
				}
			}
			if (hasGroupBy)
			{
				for (int i = 0; i < groupByFieldList.Length; i++)
				{
					name = groupByFieldList[i];
					if (!map.ContainsKey(name))
					{
						l.Add(name);
						map.Add(name, name);
					}
				}
			}
			if (HasOrderBy())
			{
				for (int i = 0; i < orderByFields.Length; i++)
				{
					name = orderByFields[i];
					if (!map.ContainsKey(name))
					{
						l.Add(name);
						map.Add(name, name);
					}
				}
			}
			map.Clear();
			map = null;
			return l;
		}

		public virtual bool IsMultiRow()
		{
			bool isMultiRow = true;
			NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction oa = null;
			// Group by protection
			// When a group by with one field exist in the query, FieldObjectAction with this field must be set to SingleRow
			bool groupBy = hasGroupBy && groupByFieldList.Length == 1;
			string oneGroupByField = null;
			System.Collections.Generic.IEnumerator<NeoDatis.Odb.Core.Query.Execution.IQueryFieldAction
				> iterator = objectActions.GetEnumerator();
			if (groupBy)
			{
				oneGroupByField = groupByFieldList[0];
				while (iterator.MoveNext())
				{
					oa = iterator.Current;
					if (oa is NeoDatis.Odb.Impl.Core.Query.Values.FieldValueAction && oa.GetAttributeName
						().Equals(oneGroupByField))
					{
						oa.SetMultiRow(false);
					}
				}
			}
			iterator = objectActions.GetEnumerator();
			if (iterator.MoveNext())
			{
				oa = iterator.Current;
				isMultiRow = oa.IsMultiRow();
			}
			while (iterator.MoveNext())
			{
				oa = iterator.Current;
				if (isMultiRow != oa.IsMultiRow())
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ValuesQueryNotConsistent
						.AddParameter(this));
				}
			}
			return isMultiRow;
		}

		public virtual NeoDatis.Odb.Core.Query.IValuesQuery GroupBy(string fieldList)
		{
			groupByFieldList = NeoDatis.Tool.Wrappers.OdbString.Split(fieldList, ",");
			hasGroupBy = true;
			return this;
		}

		public virtual bool HasGroupBy()
		{
			return hasGroupBy;
		}

		public virtual string[] GetGroupByFieldList()
		{
			return groupByFieldList;
		}

		public virtual bool ReturnInstance()
		{
			return returnInstance;
		}

		public virtual void SetReturnInstance(bool returnInstance)
		{
			this.returnInstance = returnInstance;
		}
	}
}
