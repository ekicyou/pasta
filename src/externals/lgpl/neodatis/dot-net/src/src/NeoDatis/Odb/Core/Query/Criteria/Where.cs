namespace NeoDatis.Odb.Core.Query.Criteria
{
	/// <summary>A simple factory to build all Criterion and Expression</summary>
	/// <author>olivier s</author>
	public class Where
	{
		internal Where()
		{
		}

		/// <param name="attributeName">The attribute name</param>
		/// <param name="value">The boolean value</param>
		/// <returns>The criteria</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, bool value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				 ? true : false);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Equal(string attributeName
			, object value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Iequal(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				, false);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Iequal(string attributeName
			, object value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
				, false);
		}

		/// <summary>LIKE</summary>
		/// <param name="attributeName">The attribute name</param>
		/// <param name="value">The string value</param>
		/// <returns>The criterio</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Like(string attributeName
			, string value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.LikeCriterion(attributeName, value
				, true);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ilike(string attributeName
			, string value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.LikeCriterion(attributeName, value
				, false);
		}

		/// <summary>GREATER THAN</summary>
		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		/// <returns>The criterion</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, System.IComparable value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Gt(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		/// <summary>GREATER OR EQUAL</summary>
		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		/// <returns>The criterion</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, System.IComparable value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Ge(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
		}

		/// <summary>LESS THAN</summary>
		/// <param name="attributeName"></param>
		/// <param name="value"></param>
		/// <returns>The criterion</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, System.IComparable value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Lt(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
		}

		/// <summary>LESS OR EQUAL</summary>
		/// <param name="attributeName">The attribute name</param>
		/// <param name="value">The value</param>
		/// <returns>The criterion</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, System.IComparable value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Le(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
				, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
		}

		/// <summary>The</summary>
		/// <param name="attributeName">The attribute name</param>
		/// <param name="value">The value</param>
		/// <returns>The criterion</returns>
		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, bool value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value ? true : false);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, int value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, short value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, byte value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, float value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, double value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, long value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, char value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Contain(string attributeName
			, object value)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
				value);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion IsNull(string attributeName
			)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.IsNullCriterion(attributeName);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion IsNotNull(string attributeName
			)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.IsNotNullCriterion(attributeName
				);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeEq(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeEq);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeNe(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeNe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeGt(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeGt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeGe(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeGe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeLt(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeLt);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion SizeLe(string attributeName
			, int size)
		{
			return new NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion(attributeName
				, size, NeoDatis.Odb.Impl.Core.Query.Criteria.CollectionSizeCriterion.SizeLe);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.Or Or()
		{
			return new NeoDatis.Odb.Core.Query.Criteria.Or();
		}

		public static NeoDatis.Odb.Core.Query.Criteria.And And()
		{
			return new NeoDatis.Odb.Core.Query.Criteria.And();
		}

		public static NeoDatis.Odb.Core.Query.Criteria.Not Not(NeoDatis.Odb.Core.Query.Criteria.ICriterion
			 criterion)
		{
			return new NeoDatis.Odb.Core.Query.Criteria.Not(criterion);
		}

		public static NeoDatis.Odb.Core.Query.Criteria.ICriterion Get(string attributeName
			, NeoDatis.Odb.Core.Query.Criteria.Operator @operator, string value)
		{
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.Equal)
			{
				return new NeoDatis.Odb.Impl.Core.Query.Criteria.EqualCriterion(attributeName, value
					);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.Like)
			{
				return new NeoDatis.Odb.Impl.Core.Query.Criteria.LikeCriterion(attributeName, value
					, true);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.GreaterOrEqual)
			{
				return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
					, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGe);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.GreaterThan)
			{
				return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
					, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeGt);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.LessThan)
			{
				return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
					, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLt);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.LessOrEqual)
			{
				return new NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion(attributeName, value
					, NeoDatis.Odb.Core.Query.Criteria.ComparisonCriterion.ComparisonTypeLe);
			}
			if (@operator == NeoDatis.Odb.Core.Query.Criteria.Operator.Contain)
			{
				return new NeoDatis.Odb.Impl.Core.Query.Criteria.ContainsCriterion(attributeName, 
					value);
			}
			throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.QueryUnknownOperator
				.AddParameter(@operator.GetName()));
		}
	}
}
