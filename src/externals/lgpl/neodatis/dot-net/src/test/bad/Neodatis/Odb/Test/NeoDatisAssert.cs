namespace NeoDatis.Odb.Test
{
	/// <author>olivier</author>
	public class NeoDatisAssert
	{
		/// <summary>Asserts that a condition is true.</summary>
		/// <remarks>
		/// Asserts that a condition is true. If it isn't it throws
		/// an AssertionFailedError with the given message.
		/// </remarks>
		public virtual void AssertTrue(string message, bool condition)
		{
			if (!condition)
			{
				Fail(message);
			}
		}

		/// <summary>Asserts that a condition is true.</summary>
		/// <remarks>
		/// Asserts that a condition is true. If it isn't it throws
		/// an AssertionFailedError.
		/// </remarks>
		public virtual void AssertTrue(bool condition)
		{
			AssertTrue(null, condition);
		}

		/// <summary>Asserts that a condition is false.</summary>
		/// <remarks>
		/// Asserts that a condition is false. If it isn't it throws
		/// an AssertionFailedError with the given message.
		/// </remarks>
		public virtual void AssertFalse(string message, bool condition)
		{
			AssertTrue(message, !condition);
		}

		/// <summary>Asserts that a condition is false.</summary>
		/// <remarks>
		/// Asserts that a condition is false. If it isn't it throws
		/// an AssertionFailedError.
		/// </remarks>
		public virtual void AssertFalse(bool condition)
		{
			AssertFalse(null, condition);
		}

		/// <summary>Fails a test with the given message.</summary>
		/// <remarks>Fails a test with the given message.</remarks>
		public virtual void Fail(string message)
		{
			throw new System.Exception(string.Format("%s", message));
		}

		/// <summary>Fails a test with no message.</summary>
		/// <remarks>Fails a test with no message.</remarks>
		public virtual void Fail()
		{
			Fail(null);
		}

		/// <summary>Asserts that two objects are equal.</summary>
		/// <remarks>
		/// Asserts that two objects are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, object expected, object actual)
		{
			if (expected == null && actual == null)
			{
				return;
			}
			if (expected != null && expected.Equals(actual))
			{
				return;
			}
			FailNotEquals(message, expected, actual);
		}

		/// <summary>Asserts that two objects are equal.</summary>
		/// <remarks>
		/// Asserts that two objects are equal. If they are not
		/// an AssertionFailedError is thrown.
		/// </remarks>
		public virtual void AssertEquals(object expected, object actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two Strings are equal.</summary>
		/// <remarks>Asserts that two Strings are equal.</remarks>
		public virtual void AssertEquals(string message, string expected, string actual)
		{
			if (expected == null && actual == null)
			{
				return;
			}
			if (expected != null && expected.Equals(actual))
			{
				return;
			}
			throw new System.Exception(string.Format("%s : expected=%s, actual=%s", message, 
				expected, actual));
		}

		/// <summary>Asserts that two Strings are equal.</summary>
		/// <remarks>Asserts that two Strings are equal.</remarks>
		public virtual void AssertEquals(string expected, string actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two doubles are equal concerning a delta.</summary>
		/// <remarks>
		/// Asserts that two doubles are equal concerning a delta.  If they are not
		/// an AssertionFailedError is thrown with the given message.  If the expected
		/// value is infinity then the delta value is ignored.
		/// </remarks>
		public virtual void AssertEquals(string message, double expected, double actual, 
			double delta)
		{
			if (expected==actual)
			{
				return;
			}
			if (!(System.Math.Abs(expected - actual) <= delta))
			{
				FailNotEquals(message, expected, actual);
			}
		}

		/// <summary>Asserts that two doubles are equal concerning a delta.</summary>
		/// <remarks>
		/// Asserts that two doubles are equal concerning a delta. If the expected
		/// value is infinity then the delta value is ignored.
		/// </remarks>
		public virtual void AssertEquals(double expected, double actual, double delta)
		{
			AssertEquals(null, expected, actual, delta);
		}

		/// <summary>Asserts that two floats are equal concerning a delta.</summary>
		/// <remarks>
		/// Asserts that two floats are equal concerning a delta. If they are not
		/// an AssertionFailedError is thrown with the given message.  If the expected
		/// value is infinity then the delta value is ignored.
		/// </remarks>
		public virtual void AssertEquals(string message, float expected, float actual, float
			 delta)
		{
			// handle infinity specially since subtracting to infinite values gives NaN and the
			// the following test fails
			if (false)//float.IsInfinite(expected))
			{
				if (!(expected == actual))
				{
					FailNotEquals(message, expected, actual);
				}
			}
			else
			{
				if (!(System.Math.Abs(expected - actual) <= delta))
				{
					FailNotEquals(message, expected, actual);
				}
			}
		}

		/// <summary>Asserts that two floats are equal concerning a delta.</summary>
		/// <remarks>
		/// Asserts that two floats are equal concerning a delta. If the expected
		/// value is infinity then the delta value is ignored.
		/// </remarks>
		public virtual void AssertEquals(float expected, float actual, float delta)
		{
			AssertEquals(null, expected, actual, delta);
		}

		/// <summary>Asserts that two longs are equal.</summary>
		/// <remarks>
		/// Asserts that two longs are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, long expected, long actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two longs are equal.</summary>
		/// <remarks>Asserts that two longs are equal.</remarks>
		public virtual void AssertEquals(long expected, long actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two booleans are equal.</summary>
		/// <remarks>
		/// Asserts that two booleans are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, bool expected, bool actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two booleans are equal.</summary>
		/// <remarks>Asserts that two booleans are equal.</remarks>
		public virtual void AssertEquals(bool expected, bool actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two bytes are equal.</summary>
		/// <remarks>
		/// Asserts that two bytes are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, byte expected, byte actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two bytes are equal.</summary>
		/// <remarks>Asserts that two bytes are equal.</remarks>
		public virtual void AssertEquals(byte expected, byte actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two chars are equal.</summary>
		/// <remarks>
		/// Asserts that two chars are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, char expected, char actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two chars are equal.</summary>
		/// <remarks>Asserts that two chars are equal.</remarks>
		public virtual void AssertEquals(char expected, char actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two shorts are equal.</summary>
		/// <remarks>
		/// Asserts that two shorts are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, short expected, short actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two shorts are equal.</summary>
		/// <remarks>Asserts that two shorts are equal.</remarks>
		public virtual void AssertEquals(short expected, short actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that two ints are equal.</summary>
		/// <remarks>
		/// Asserts that two ints are equal. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertEquals(string message, int expected, int actual)
		{
			AssertEquals(message, expected, actual);
		}

		/// <summary>Asserts that two ints are equal.</summary>
		/// <remarks>Asserts that two ints are equal.</remarks>
		public virtual void AssertEquals(int expected, int actual)
		{
			AssertEquals(null, expected, actual);
		}

		/// <summary>Asserts that an object isn't null.</summary>
		/// <remarks>Asserts that an object isn't null.</remarks>
		public virtual void AssertNotNull(object @object)
		{
			AssertNotNull(null, @object);
		}

		/// <summary>Asserts that an object isn't null.</summary>
		/// <remarks>
		/// Asserts that an object isn't null. If it is
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertNotNull(string message, object @object)
		{
			AssertTrue(message, @object != null);
		}

		/// <summary>Asserts that an object is null.</summary>
		/// <remarks>Asserts that an object is null.</remarks>
		public virtual void AssertNull(object @object)
		{
			AssertNull(null, @object);
		}

		/// <summary>Asserts that an object is null.</summary>
		/// <remarks>
		/// Asserts that an object is null.  If it is not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertNull(string message, object @object)
		{
			AssertTrue(message, @object == null);
		}

		/// <summary>Asserts that two objects refer to the same object.</summary>
		/// <remarks>
		/// Asserts that two objects refer to the same object. If they are not
		/// an AssertionFailedError is thrown with the given message.
		/// </remarks>
		public virtual void AssertSame(string message, object expected, object actual)
		{
			if (expected == actual)
			{
				return;
			}
			FailNotSame(message, expected, actual);
		}

		/// <summary>Asserts that two objects refer to the same object.</summary>
		/// <remarks>
		/// Asserts that two objects refer to the same object. If they are not
		/// the same an AssertionFailedError is thrown.
		/// </remarks>
		public virtual void AssertSame(object expected, object actual)
		{
			AssertSame(null, expected, actual);
		}

		/// <summary>Asserts that two objects do not refer to the same object.</summary>
		/// <remarks>
		/// Asserts that two objects do not refer to the same object. If they do
		/// refer to the same object an AssertionFailedError is thrown with the
		/// given message.
		/// </remarks>
		public virtual void AssertNotSame(string message, object expected, object actual)
		{
			if (expected == actual)
			{
				FailSame(message);
			}
		}

		/// <summary>Asserts that two objects do not refer to the same object.</summary>
		/// <remarks>
		/// Asserts that two objects do not refer to the same object. If they do
		/// refer to the same object an AssertionFailedError is thrown.
		/// </remarks>
		public virtual void AssertNotSame(object expected, object actual)
		{
			AssertNotSame(null, expected, actual);
		}

		public virtual void FailSame(string message)
		{
			string formatted = string.Empty;
			if (message != null)
			{
				formatted = message + " ";
			}
			Fail(formatted + "expected not same");
		}

		public virtual void FailNotSame(string message, object expected, object actual)
		{
			string formatted = string.Empty;
			if (message != null)
			{
				formatted = message + " ";
			}
			Fail(formatted + "expected same:<" + expected + "> was not:<" + actual + ">");
		}

		public virtual void FailNotEquals(string message, object expected, object actual)
		{
			Fail(Format(message, expected, actual));
		}

		internal static string Format(string message, object expected, object actual)
		{
			string formatted = string.Empty;
			if (message != null)
			{
				formatted = message + " ";
			}
			return formatted + "expected:<" + expected + "> but was:<" + actual + ">";
		}

		public virtual string GetName()
		{
			return "my name";
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void SetUp()
		{
		}

		/// <exception cref="System.Exception"></exception>
		public virtual void TearDown()
		{
		}
	}
}
