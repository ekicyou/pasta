using NeoDatis.Odb.Test.VO.Arraycollectionmap;
using NeoDatis.Odb.Impl.Core.Query.Criteria;
using NeoDatis.Odb.Core.Query.Criteria;
namespace NeoDatis.Odb.Test.Arraycollectionmap
{
	public class TestArray : NeoDatis.Odb.Test.ODBTest
	{
		/// <exception cref="System.Exception"></exception>
		public virtual void TestArray1()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				decimal nb = odb.Count(new CriteriaQuery(typeof(
					PlayerWithArray)));
				PlayerWithArray player = new PlayerWithArray
					("kiko");
				player.AddGame("volley-ball");
				player.AddGame("squash");
				player.AddGame("tennis");
				player.AddGame("ping-pong");
				odb.Store(player);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<PlayerWithArray> l = odb.GetObjects<PlayerWithArray>(true);
				AssertEquals(nb + 1, l.Count);
				// gets first player
				PlayerWithArray player2 = l.GetFirst();
				AssertEquals(player.ToString(), player2.ToString());
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestArray2()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				int[] intArray = new int[size];
				for (int i = 0; i < size; i++)
				{
					intArray[i] = i;
				}
				ObjectWithNativeArrayOfInt owna = new ObjectWithNativeArrayOfInt
					("t1", intArray);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfInt> l = odb.GetObjects<ObjectWithNativeArrayOfInt>();
				ObjectWithNativeArrayOfInt owna2 = l.GetFirst();
				AssertEquals(owna.GetName(), owna2.GetName());
				for (int i = 0; i < size; i++)
				{
					AssertEquals(owna.GetNumbers()[i], owna2.GetNumbers()[i]);
				}
				odb.Close();
				odb = null;
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestArray3()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				short[] array = new short[size];
				for (int i = 0; i < size; i++)
				{
					array[i] = (short)i;
				}
				ObjectWithNativeArrayOfShort owna = new ObjectWithNativeArrayOfShort
					("t1", array);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfShort> l = odb.GetObjects<ObjectWithNativeArrayOfShort>();
				ObjectWithNativeArrayOfShort owna2 = l.GetFirst();
				AssertEquals(owna.GetName(), owna2.GetName());
				for (int i = 0; i < size; i++)
				{
					AssertEquals(owna.GetNumbers()[i], owna2.GetNumbers()[i]);
				}
				odb.Close();
				odb = null;
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestArrayQuery()
		{
			NeoDatis.Odb.ODB odb = null;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				decimal nb = odb.Count(new CriteriaQuery(typeof(
					PlayerWithArray)));
				PlayerWithArray player = new PlayerWithArray
					("kiko");
				player.AddGame("volley-ball");
				player.AddGame("squash");
				player.AddGame("tennis");
				player.AddGame("ping-pong");
				odb.Store(player);
				odb.Close();
				odb = Open("array1.neodatis");
                NeoDatis.Odb.Objects<PlayerWithArray> l = odb.GetObjects < PlayerWithArray>(new CriteriaQuery(Where.Contain("games", "tennis")));
				AssertEquals(nb + 1, l.Count);
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestArray4()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				odb = Open("array1.neodatis");
				System.Decimal[] array = new System.Decimal[size];
				for (int i = 0; i < size; i++)
				{
					array[i] = new System.Decimal(((double)i) * 78954545 / 89);
				}
				ObjectWithNativeArrayOfBigDecimal owna = new 
					ObjectWithNativeArrayOfBigDecimal("t1", array
					);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfBigDecimal> l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal owna2 = l.GetFirst();
				AssertEquals(owna.GetName(), owna2.GetName());
				for (int i = 0; i < size; i++)
				{
					AssertEquals(owna.GetNumbers()[i], owna2.GetNumbers()[i]);
				}
				odb.Close();
				odb = null;
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TestArrayOfDate()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				System.DateTime[] array = new System.DateTime[size];
				System.DateTime now = new System.DateTime();
				for (int i = 0; i < size; i++)
				{
					array[i] = new System.DateTime(now.Millisecond + i);
				}
				ObjectWithNativeArrayOfDate owna = new ObjectWithNativeArrayOfDate
					("t1", array);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfDate> l = odb.GetObjects<ObjectWithNativeArrayOfDate>();
				ObjectWithNativeArrayOfDate owna2 = l.GetFirst();
				AssertEquals(owna.GetName(), owna2.GetName());
				for (int i = 0; i < size; i++)
				{
					AssertEquals(owna.GetNumbers()[i], owna2.GetNumbers()[i]);
				}
				odb.Close();
				odb = null;
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <summary>
		/// Test in place update for array when the number of elements remains the
		/// same
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestArray5()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				System.Decimal[] array = new System.Decimal[size];
				for (int i = 0; i < size; i++)
				{
					array[i] = new System.Decimal(((double)i) * 78954545 / 89);
				}
				ObjectWithNativeArrayOfBigDecimal owna = new 
					ObjectWithNativeArrayOfBigDecimal("t1", array
					);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfBigDecimal> l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal owna2 = l.GetFirst();
				owna2.SetNumber(0, new System.Decimal(1));
				odb.Store(owna2);
				odb.Close();
				odb = Open("array1.neodatis");
				l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal o = l.GetFirst();
				AssertEquals(owna2.GetNumber(0), o.GetNumber(0));
				AssertEquals(owna2.GetNumber(1), o.GetNumber(1));
				if (isLocal)
				{
					// check that it was in place update and not normal update (by
					// creatig now object)
					AssertEquals(0, NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.
						GetNbInPlaceUpdates());
					AssertEquals(1, NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.
						GetNbNormalUpdates());
				}
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <summary>
		/// Test in place update for array when the number of elements remains the
		/// same
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestArray6()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 2;
			try
			{
				NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				int[] array = new int[size];
				for (int i = 0; i < size; i++)
				{
					array[i] = i;
				}
				ObjectWithNativeArrayOfInt owna = new ObjectWithNativeArrayOfInt("t1", array);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfInt> l = odb.GetObjects<ObjectWithNativeArrayOfInt>();
				ObjectWithNativeArrayOfInt owna2 = l.GetFirst();
				owna2.SetNumber(0, 1);
				odb.Store(owna2);
				odb.Close();
				odb = Open("array1.neodatis");
				l = odb.GetObjects<ObjectWithNativeArrayOfInt>();
				ObjectWithNativeArrayOfInt o = l.GetFirst();
				AssertEquals(1, o.GetNumber(0));
				AssertEquals(1, o.GetNumber(1));
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <summary>
		/// Test in place update for array when the number of elements remains the
		/// same,but updating the second array element
		/// </summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestArray61()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.AbstractObjectWriter.ResetNbUpdates();
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				int[] array = new int[size];
				for (int i = 0; i < size; i++)
				{
					array[i] = i;
				}
				ObjectWithNativeArrayOfInt owna = new ObjectWithNativeArrayOfInt
					("t1", array);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
                NeoDatis.Odb.Objects<ObjectWithNativeArrayOfInt> l = odb.GetObjects<ObjectWithNativeArrayOfInt>();
				ObjectWithNativeArrayOfInt owna2 = (ObjectWithNativeArrayOfInt
					)l.GetFirst();
				owna2.SetNumber(1, 78);
				odb.Store(owna2);
				odb.Close();
				odb = Open("array1.neodatis");
				l = odb.GetObjects<ObjectWithNativeArrayOfInt>();
				ObjectWithNativeArrayOfInt o = l.GetFirst();
				AssertEquals(0, o.GetNumber(0));
				AssertEquals(78, o.GetNumber(1));
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <summary>Increasing array size</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestArray6UpdateIncreasingArraySize()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				System.Decimal[] array = new System.Decimal[size];
				System.Decimal[] array2 = new System.Decimal[size + 1];
				for (int i = 0; i < size; i++)
				{
					array[i] = new System.Decimal(((double)i) * 78954545 / 89);
					array2[i] = new System.Decimal(((double)i) * 78954545 / 89);
				}
				array2[size] = new System.Decimal(100);
				ObjectWithNativeArrayOfBigDecimal owna = new 
					ObjectWithNativeArrayOfBigDecimal("t1", array
					);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfBigDecimal> l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal owna2 = l.GetFirst();
				owna2.SetNumbers(array2);
				odb.Store(owna2);
				odb.Close();
				odb = Open("array1.neodatis");
				l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal o = l.GetFirst();
				AssertEquals(size + 1, o.GetNumbers().Length);
				AssertEquals(new System.Decimal(100), o.GetNumber(size));
				AssertEquals(owna2.GetNumber(1), o.GetNumber(1));
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}

		/// <summary>Decreasing array size</summary>
		/// <exception cref="System.Exception">System.Exception</exception>
		public virtual void TestArrayUpdateDecreasingArraySize()
		{
			NeoDatis.Odb.ODB odb = null;
			int size = 50;
			try
			{
				DeleteBase("array1.neodatis");
				odb = Open("array1.neodatis");
				System.Decimal[] array = new System.Decimal[size];
				System.Decimal[] array2 = new System.Decimal[size + 1];
				for (int i = 0; i < size; i++)
				{
					array[i] = new System.Decimal(((double)i) * 78954545 / 89);
					array2[i] = new System.Decimal(((double)i) * 78954545 / 89);
				}
				array[size - 1] = new System.Decimal(99);
				array2[size] = new System.Decimal(100);
				ObjectWithNativeArrayOfBigDecimal owna = new 
					ObjectWithNativeArrayOfBigDecimal("t1", array2
					);
				odb.Store(owna);
				odb.Close();
				odb = Open("array1.neodatis");
				NeoDatis.Odb.Objects<ObjectWithNativeArrayOfBigDecimal> l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal owna2 = l.GetFirst();
				owna2.SetNumbers(array);
				odb.Store(owna2);
				odb.Close();
				odb = Open("array1.neodatis");
				l = odb.GetObjects<ObjectWithNativeArrayOfBigDecimal>();
				ObjectWithNativeArrayOfBigDecimal o = (ObjectWithNativeArrayOfBigDecimal
					)l.GetFirst();
				AssertEquals(size, o.GetNumbers().Length);
				AssertEquals(new System.Decimal(99), o.GetNumber(size - 1));
				AssertEquals(owna2.GetNumber(1), o.GetNumber(1));
				odb = null;
			}
			catch (System.Exception e)
			{
				if (odb != null)
				{
					odb.Rollback();
					odb = null;
				}
				throw;
			}
			finally
			{
				if (odb != null)
				{
					odb.Close();
				}
				DeleteBase("array1.neodatis");
			}
		}
	}
}
