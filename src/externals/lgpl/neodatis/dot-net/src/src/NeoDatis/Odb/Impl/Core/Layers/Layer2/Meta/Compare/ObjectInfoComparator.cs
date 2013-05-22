using NeoDatis.Odb.Core.Layers.Layer2.Meta;
using NeoDatis.Odb.Core.Layers.Layer2.Meta.Compare;
using System.Collections.Generic;
using NeoDatis.Tool.Wrappers.Map;
namespace NeoDatis.Odb.Impl.Core.Layers.Layer2.Meta.Compare
{
	/// <summary>Manage Object info differences.</summary>
	/// <remarks>
	/// Manage Object info differences. compares two object info and tells which
	/// objects in the object hierarchy has changed. This is used by the update to process to optimize it and actually update what has changed
	/// </remarks>
	/// <author>olivier s</author>
	public class ObjectInfoComparator : IObjectInfoComparator
	{
		private const int Size = 5;

		public IList<NonNativeObjectInfo> changedObjectMetaRepresentations;

		private IList<SetAttributeToNullAction> attributeToSetToNull;

		public IList<ChangedAttribute> changedAttributeActions;

		private IDictionary<NonNativeObjectInfo, int> alreadyCheckingObjects;

		public IList<object> newObjects;

		public IList<NewNonNativeObjectAction> newObjectMetaRepresentations;

		public IList<ChangedObjectInfo> changes;

		private IList<ArrayModifyElement> arrayChanges;

		public int maxObjectRecursionLevel;

		private int nbChanges;

		private bool supportInPlaceUpdate;

		public ObjectInfoComparator()
		{
			changedObjectMetaRepresentations = new List<NonNativeObjectInfo>(Size);
			attributeToSetToNull = new List<SetAttributeToNullAction>(Size);
			alreadyCheckingObjects = new OdbHashMap<NonNativeObjectInfo	, int>(Size);
			newObjects = new List<object>(Size);
			newObjectMetaRepresentations = new List<NewNonNativeObjectAction>(Size);
			changes = new List<ChangedObjectInfo>(Size);
			changedAttributeActions = new List<ChangedAttribute>(Size);
			arrayChanges = new List<ArrayModifyElement>();
			maxObjectRecursionLevel = 0;
			supportInPlaceUpdate = false;
		}

		public virtual bool HasChanged(AbstractObjectInfo
			 aoi1, AbstractObjectInfo aoi2)
		{
			return HasChanged(aoi1, aoi2, -1);
		}

		private bool HasChanged(AbstractObjectInfo aoi1
			, AbstractObjectInfo aoi2, int objectRecursionLevel
			)
		{
			// If one is null and the other not
			if (aoi1.IsNull() != aoi2.IsNull())
			{
				return true;
			}
			if (aoi1.IsNonNativeObject() && aoi2.IsNonNativeObject())
			{
				return HasChanged((NonNativeObjectInfo)aoi1, 
					(NonNativeObjectInfo)aoi2, objectRecursionLevel
					 + 1);
			}
			if (aoi1.IsNative() && aoi2.IsNative())
			{
				return HasChanged((NativeObjectInfo)aoi1, (NativeObjectInfo
					)aoi2, 0);
			}
			return false;
		}

		private bool HasChanged(NativeObjectInfo aoi1
			, NativeObjectInfo aoi2, int objectRecursionLevel
			)
		{
			if (aoi1.GetObject() == null && aoi2.GetObject() == null)
			{
				return false;
			}
			if (aoi1.GetObject() == null || aoi2.GetObject() == null)
			{
				return true;
			}
			return !aoi1.GetObject().Equals(aoi2.GetObject());
		}

		private bool HasChanged(NonNativeObjectInfo nnoi1, NonNativeObjectInfo nnoi2, int objectRecursionLevel)
		{
			AbstractObjectInfo value1 = null;
			AbstractObjectInfo value2 = null;
			bool hasChanged = false;
			// If the object is already being checked, return false, this second
			// check will not affect the check
            int n = 0;
            alreadyCheckingObjects.TryGetValue(nnoi2, out n);
			if (n != 0)
			{
				return false;
			}
			// Put the object in the temporary cache
			alreadyCheckingObjects[nnoi1] = 1;
			alreadyCheckingObjects[nnoi2] = 1;
			// Warning ID Start with 1 and not 0
			for (int id = 1; id <= nnoi1.GetMaxNbattributes(); id++)
			{
				value1 = nnoi1.GetAttributeValueFromId(id);
				// Gets the value by the attribute id to be sure
				// Problem because a new object info may not have the right ids ?
				// Check if
				// the new oiD is ok.
				value2 = nnoi2.GetAttributeValueFromId(id);
				if (value2 == null)
				{
					// this means the object to have attribute id
					StoreChangedObject(nnoi1, nnoi2, id, objectRecursionLevel);
					hasChanged = true;
					continue;
				}
				if (value1 == null)
				{
					//throw new ODBRuntimeException("ObjectInfoComparator.hasChanged:attribute with id "+id+" does not exist on "+nnoi2);
					// This happens when this object was created with an version of ClassInfo (which has been refactored).
					// In this case,we simply tell that in place update is not supported so that the object will be rewritten with 
					// new metamodel
					supportInPlaceUpdate = false;
					continue;
				}
				// If both are null, no effect
				if (value1.IsNull() && value2.IsNull())
				{
					continue;
				}
				if (value1.IsNull() || value2.IsNull())
				{
					supportInPlaceUpdate = false;
					hasChanged = true;
					StoreActionSetAttributetoNull(nnoi1, id, objectRecursionLevel);
					continue;
				}
				if (!ClassAreCompatible(value1, value2))
				{
					if (value2 is NativeObjectInfo)
					{
						StoreChangedObject(nnoi1, nnoi2, id, objectRecursionLevel);
						StoreChangedAttributeAction(new ChangedNativeAttributeAction
							(nnoi1, nnoi2, nnoi1.GetHeader().GetAttributeIdentificationFromId(id), (NativeObjectInfo
							)value2, objectRecursionLevel, false, nnoi1.GetClassInfo().GetAttributeInfoFromId
							(id).GetName()));
					}
					if (value2 is ObjectReference)
					{
						NonNativeObjectInfo nnoi = (NonNativeObjectInfo
							)value1;
						ObjectReference oref = (ObjectReference
							)value2;
						if (!nnoi.GetOid().Equals(oref.GetOid()))
						{
							StoreChangedObject(nnoi1, nnoi2, id, objectRecursionLevel);
							int attributeIdThatHasChanged = id;
							// this is the exact position where the object reference
							// definition is stored
							long attributeDefinitionPosition = nnoi2.GetAttributeDefinitionPosition(attributeIdThatHasChanged
								);
							StoreChangedAttributeAction(new ChangedObjectReferenceAttributeAction
								(attributeDefinitionPosition, (ObjectReference
								)value2, objectRecursionLevel));
						}
						else
						{
							continue;
						}
					}
					hasChanged = true;
					continue;
				}
				if (value1.IsAtomicNativeObject())
				{
					if (!value1.Equals(value2))
					{
						// storeChangedObject(nnoi1, nnoi2, id,
						// objectRecursionLevel);
						StoreChangedAttributeAction(new ChangedNativeAttributeAction
							(nnoi1, nnoi2, nnoi1.GetHeader().GetAttributeIdentificationFromId(id), (NativeObjectInfo
							)value2, objectRecursionLevel, false, nnoi1.GetClassInfo().GetAttributeInfoFromId
							(id).GetName()));
						hasChanged = true;
						continue;
					}
					continue;
				}
				if (value1.IsCollectionObject())
				{
					CollectionObjectInfo coi1 = (CollectionObjectInfo)value1;
					CollectionObjectInfo coi2 = (CollectionObjectInfo)value2;
					bool collectionHasChanged = ManageCollectionChanges(nnoi1, nnoi2, id, coi1, coi2, objectRecursionLevel);
					hasChanged = hasChanged || collectionHasChanged;
					continue;
				}
				if (value1.IsArrayObject())
				{
					ArrayObjectInfo aoi1 = (ArrayObjectInfo)value1;
					ArrayObjectInfo aoi2 = (ArrayObjectInfo)value2;
					bool arrayHasChanged = ManageArrayChanges(nnoi1, nnoi2, id, aoi1, aoi2, objectRecursionLevel
						);
					hasChanged = hasChanged || arrayHasChanged;
					continue;
				}
				if (value1.IsMapObject())
				{
					MapObjectInfo moi1 = (MapObjectInfo)value1;
					MapObjectInfo moi2 = (MapObjectInfo)value2;
					bool mapHasChanged = ManageMapChanges(nnoi1, nnoi2, id, moi1, moi2, objectRecursionLevel
						);
					hasChanged = hasChanged || mapHasChanged;
					continue;
				}
				if (value1.IsEnumObject())
				{
					EnumNativeObjectInfo enoi1 = (EnumNativeObjectInfo)value1;
					EnumNativeObjectInfo enoi2 = (EnumNativeObjectInfo)value2;
					bool enumHasChanged = !enoi1.GetEnumClassInfo().GetId().Equals(enoi2.GetEnumClassInfo
						().GetId()) || !enoi1.GetEnumName().Equals(enoi2.GetEnumName());
					hasChanged = hasChanged || enumHasChanged;
					continue;
				}
				if (value1.IsNonNativeObject())
				{
					NonNativeObjectInfo oi1 = (NonNativeObjectInfo)value1;
					NonNativeObjectInfo oi2 = (NonNativeObjectInfo)value2;
					// If oids are equal, they are the same objects
					if (oi1.GetOid() != null && oi1.GetOid().Equals(oi2.GetOid()))
					{
						hasChanged = HasChanged(value1, value2, objectRecursionLevel + 1) || hasChanged;
					}
					else
					{
						// This means that an object reference has changed.
						hasChanged = true;
						// keep track of the position where the reference must be
						// updated
						long positionToUpdateReference = nnoi1.GetAttributeDefinitionPosition(id);
						StoreNewObjectReference(positionToUpdateReference, oi2, objectRecursionLevel, nnoi1
							.GetClassInfo().GetAttributeInfoFromId(id).GetName());
						objectRecursionLevel++;
						// Value2 may have change too
						AddPendingVerification(value2);
					}
					continue;
				}
			}
			int i1 = (int)alreadyCheckingObjects[nnoi1];
			int i2 = (int)alreadyCheckingObjects[nnoi2];
			if (i1 != null)
			{
				i1 = i1 - 1;
			}
			if (i2 != null)
			{
				i2 = i2 - 1;
			}
			if (i1 == 0)
			{
				alreadyCheckingObjects.Remove(nnoi1);
			}
			else
			{
				alreadyCheckingObjects.Add(nnoi1, i1);
			}
			if (i2 == 0)
			{
				alreadyCheckingObjects.Remove(nnoi2);
			}
			else
			{
				alreadyCheckingObjects.Add(nnoi2, i2);
			}
			return hasChanged;
		}

		/// <summary>
		/// An object reference has changed and the new object has not been checked, so disabled in place update
		/// TODO this is not good =&gt; all reference update will be done by full update and not in place update
		/// </summary>
		/// <param name="value"></param>
		private void AddPendingVerification(AbstractObjectInfo
			 value)
		{
			supportInPlaceUpdate = false;
		}

		private void StoreNewObjectReference(long positionToUpdateReference, NonNativeObjectInfo
			 oi2, int objectRecursionLevel, string attributeName)
		{
			NewNonNativeObjectAction nnnoa = new 
				NewNonNativeObjectAction(positionToUpdateReference
				, oi2, objectRecursionLevel, attributeName);
			newObjectMetaRepresentations.Add(nnnoa);
			nbChanges++;
		}

		private void StoreActionSetAttributetoNull(NonNativeObjectInfo
			 nnoi, int id, int objectRecursionLevel)
		{
			nbChanges++;
			SetAttributeToNullAction action = new 
				SetAttributeToNullAction(nnoi, id);
			attributeToSetToNull.Add(action);
		}

		private void StoreArrayChange(NonNativeObjectInfo
			 nnoi, int arrayAttributeId, int arrayIndex, AbstractObjectInfo
			 value, bool supportInPlaceUpdate)
		{
			nbChanges++;
			ArrayModifyElement ame = new ArrayModifyElement
				(nnoi, arrayAttributeId, arrayIndex, value, supportInPlaceUpdate);
			arrayChanges.Add(ame);
		}

		private bool ClassAreCompatible(AbstractObjectInfo
			 value1, AbstractObjectInfo value2)
		{
			System.Type clazz1 = value1.GetType();
			System.Type clazz2 = value2.GetType();
			if (clazz1 == clazz2)
			{
				return true;
			}
			if ((clazz1 == typeof(NonNativeObjectInfo)) 
				&& (clazz2 == typeof(NeoDatis.Odb.Core.Server.Layers.Layer2.Meta.ClientNonNativeObjectInfo
				)))
			{
				return true;
			}
			return false;
		}

		private void StoreChangedObject(NonNativeObjectInfo
			 aoi1, NonNativeObjectInfo aoi2, int fieldId
			, AbstractObjectInfo oldValue, AbstractObjectInfo
			 newValue, string message, int objectRecursionLevel)
		{
			if (aoi1 != null && aoi2 != null)
			{
				if (aoi1.GetOid() != null && aoi1.GetOid().Equals(aoi2.GetOid()))
				{
					changedObjectMetaRepresentations.Add(aoi2);
					changes.Add(new ChangedObjectInfo(aoi1
						.GetClassInfo(), aoi2.GetClassInfo(), fieldId, oldValue, newValue, message, objectRecursionLevel
						));
					// also the max recursion level
					if (objectRecursionLevel > maxObjectRecursionLevel)
					{
						maxObjectRecursionLevel = objectRecursionLevel;
					}
					nbChanges++;
				}
				else
				{
					newObjects.Add(aoi2.GetObject());
					string fieldName = aoi1.GetClassInfo().GetAttributeInfoFromId(fieldId).GetName();
					// keep track of the position where the reference must be
					// updated - use aoi1 to get position, because aoi2 do not have position defined yet
					long positionToUpdateReference = aoi1.GetAttributeDefinitionPosition(fieldId);
					StoreNewObjectReference(positionToUpdateReference, aoi2, objectRecursionLevel, fieldName
						);
				}
			}
			else
			{
				//newObjectMetaRepresentations.add(aoi2);
				NeoDatis.Tool.DLogger.Info("Non native object with null object");
			}
		}

		private void StoreChangedObject(NonNativeObjectInfo
			 aoi1, NonNativeObjectInfo aoi2, int fieldId
			, int objectRecursionLevel)
		{
			nbChanges++;
			if (aoi1 != null && aoi2 != null)
			{
				changes.Add(new ChangedObjectInfo(aoi1
					.GetClassInfo(), aoi2.GetClassInfo(), fieldId, aoi1.GetAttributeValueFromId(fieldId
					), aoi2.GetAttributeValueFromId(fieldId), objectRecursionLevel));
				// also the max recursion level
				if (objectRecursionLevel > maxObjectRecursionLevel)
				{
					maxObjectRecursionLevel = objectRecursionLevel;
				}
			}
			else
			{
				NeoDatis.Tool.DLogger.Info("Non native object with null object");
			}
		}

		/// <summary>
		/// Checks if something in the Collection has changed, if yes, stores the
		/// change
		/// </summary>
		/// <param name="nnoi1">
		/// The first Object meta representation (nnoi =
		/// NonNativeObjectInfo)
		/// </param>
		/// <param name="nnoi2">The second object meta representation</param>
		/// <param name="fieldIndex">The field index that this collection represents</param>
		/// <param name="coi1">
		/// The Meta representation of the collection 1 (coi =
		/// CollectionObjectInfo)
		/// </param>
		/// <param name="coi2">The Meta representation of the collection 2</param>
		/// <param name="objectRecursionLevel"></param>
		/// <returns>true if 2 collection representation are different</returns>
		private bool ManageCollectionChanges(NonNativeObjectInfo
			 nnoi1, NonNativeObjectInfo nnoi2, int fieldId
			, CollectionObjectInfo coi1, CollectionObjectInfo
			 coi2, int objectRecursionLevel)
		{
			ICollection<AbstractObjectInfo
				> collection1 = coi1.GetCollection();
			ICollection<AbstractObjectInfo
				> collection2 = coi2.GetCollection();
			if (collection1.Count != collection2.Count)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder();
				buffer.Append("Collection size has changed oldsize=").Append(collection1.Count).Append
					("/newsize=").Append(collection2.Count);
				StoreChangedObject(nnoi1, nnoi2, fieldId, coi1, coi2, buffer.ToString(), objectRecursionLevel
					);
				return true;
			}
			System.Collections.IEnumerator iterator1 = collection1.GetEnumerator();
			System.Collections.IEnumerator iterator2 = collection2.GetEnumerator();
			AbstractObjectInfo value1 = null;
			AbstractObjectInfo value2 = null;
			int index = 0;
			while (iterator1.MoveNext())
			{
                iterator2.MoveNext();
                value1 = (AbstractObjectInfo)iterator1.Current;
				value2 = (AbstractObjectInfo)iterator2.Current;
				bool hasChanged = this.HasChanged(value1, value2, objectRecursionLevel);
				if (hasChanged)
				{
					// We consider collection has changed only if object are
					// different, If objects are the same instance, but something in
					// the object has changed, then the collection has not
					// changed,only the object
					if (value1.IsNonNativeObject() && value2.IsNonNativeObject())
					{
						NonNativeObjectInfo nnoia = (NonNativeObjectInfo
							)value1;
						NonNativeObjectInfo nnoib = (NonNativeObjectInfo
							)value2;
						if (nnoia.GetOid() != null && !nnoia.GetOid().Equals(nnoi2.GetOid()))
						{
							// Objects are not the same instance -> the collection
							// has changed
							StoreChangedObject(nnoi1, nnoi2, fieldId, value1, value2, "List element index " +
								 index + " has changed", objectRecursionLevel);
						}
					}
					else
					{
						supportInPlaceUpdate = false;
						nbChanges++;
					}
					//storeChangedObject(nnoi1, nnoi2, fieldId, value1, value2, "List element index " + index + " has changed", objectRecursionLevel);
					return true;
				}
				index++;
			}
			return false;
		}

		/// <summary>Checks if something in the Arary has changed, if yes, stores the change</summary>
		/// <param name="nnoi1">
		/// The first Object meta representation (nnoi =
		/// NonNativeObjectInfo)
		/// </param>
		/// <param name="nnoi2">The second object meta representation</param>
		/// <param name="fieldIndex">The field index that this collection represents</param>
		/// <param name="aoi1">The Meta representation of the array 1 (aoi = ArraybjectInfo)</param>
		/// <param name="aoi2">The Meta representation of the array 2</param>
		/// <param name="objectRecursionLevel"></param>
		/// <returns>true if the 2 array representations are different</returns>
		private bool ManageArrayChanges(NonNativeObjectInfo
			 nnoi1, NonNativeObjectInfo nnoi2, int fieldId
			, ArrayObjectInfo aoi1, ArrayObjectInfo
			 aoi2, int objectRecursionLevel)
		{
			object[] array1 = aoi1.GetArray();
			object[] array2 = aoi2.GetArray();
			if (array1.Length != array2.Length)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder();
				buffer.Append("Array size has changed oldsize=").Append(array1.Length).Append("/newsize="
					).Append(array2.Length);
				StoreChangedObject(nnoi1, nnoi2, fieldId, aoi1, aoi2, buffer.ToString(), objectRecursionLevel
					);
				supportInPlaceUpdate = false;
				return true;
			}
			AbstractObjectInfo value1 = null;
			AbstractObjectInfo value2 = null;
			// check if this array supports in place update
			bool localSupportInPlaceUpdate = ODBType.HasFixSize
				(aoi2.GetComponentTypeId());
			int index = 0;
			bool hasChanged = false;
			try
			{
				for (int i = 0; i < array1.Length; i++)
				{
					value1 = (AbstractObjectInfo)array1[i];
					value2 = (AbstractObjectInfo)array2[i];
					bool localHasChanged = this.HasChanged(value1, value2, objectRecursionLevel);
					if (localHasChanged)
					{
						StoreArrayChange(nnoi1, fieldId, i, value2, localSupportInPlaceUpdate);
						if (localSupportInPlaceUpdate)
						{
							hasChanged = true;
						}
						else
						{
							hasChanged = true;
							return hasChanged;
						}
					}
					index++;
				}
			}
			finally
			{
				if (hasChanged && !localSupportInPlaceUpdate)
				{
					supportInPlaceUpdate = false;
				}
			}
			return hasChanged;
		}

		/// <summary>Checks if something in the Map has changed, if yes, stores the change</summary>
		/// <param name="nnoi1">
		/// The first Object meta representation (nnoi =
		/// NonNativeObjectInfo)
		/// </param>
		/// <param name="nnoi2">The second object meta representation</param>
		/// <param name="fieldIndex">The field index that this map represents</param>
		/// <param name="moi1">The Meta representation of the map 1 (moi = MapObjectInfo)</param>
		/// <param name="moi2">The Meta representation of the map 2</param>
		/// <param name="objectRecursionLevel"></param>
		/// <returns>true if the 2 map representations are different</returns>
		private bool ManageMapChanges(NonNativeObjectInfo
			 nnoi1, NonNativeObjectInfo nnoi2, int fieldId
			, MapObjectInfo moi1, MapObjectInfo
			 moi2, int objectRecursionLevel)
		{
            if (true)
            {
                return true;
            }
			IDictionary<AbstractObjectInfo
				, AbstractObjectInfo> map1 = moi1.GetMap();
			IDictionary<AbstractObjectInfo
				, AbstractObjectInfo> map2 = moi2.GetMap();
			if (map1.Count != map2.Count)
			{
				System.Text.StringBuilder buffer = new System.Text.StringBuilder();
				buffer.Append("Map size has changed oldsize=").Append(map1.Count).Append("/newsize="
					).Append(map2.Count);
				StoreChangedObject(nnoi1, nnoi2, fieldId, moi1, moi2, buffer.ToString(), objectRecursionLevel
					);
				return true;
			}
			IEnumerator<AbstractObjectInfo
				> keys1 = map1.Keys.GetEnumerator();
			IEnumerator<AbstractObjectInfo
				> keys2 = map2.Keys.GetEnumerator();
			AbstractObjectInfo key1 = null;
			AbstractObjectInfo key2 = null;
			AbstractObjectInfo value1 = null;
			AbstractObjectInfo value2 = null;
			int index = 0;
			while (keys1.MoveNext())
			{
                keys2.MoveNext();
				key1 = keys1.Current;
				key2 = keys2.Current;
				bool keysHaveChanged = this.HasChanged(key1, key2, objectRecursionLevel);
				if (keysHaveChanged)
				{
					StoreChangedObject(nnoi1, nnoi2, fieldId, key1, key2, "Map key index " + index + 
						" has changed", objectRecursionLevel);
					return true;
				}
				value1 = map1[key1];
				value2 = map2[key2];
				bool valuesHaveChanged = this.HasChanged(value1, value2, objectRecursionLevel);
				if (valuesHaveChanged)
				{
					StoreChangedObject(nnoi1, nnoi2, fieldId, value1, value2, "Map value index " + index
						 + " has changed", objectRecursionLevel);
					return true;
				}
				index++;
			}
			return false;
		}

		protected virtual void StoreChangedAttributeAction(ChangedNativeAttributeAction
			 caa)
		{
			nbChanges++;
			changedAttributeActions.Add(caa);
		}

		protected virtual void StoreChangedAttributeAction(ChangedObjectReferenceAttributeAction
			 caa)
		{
			nbChanges++;
			changedAttributeActions.Add(caa);
		}

		public virtual AbstractObjectInfo GetChangedObjectMetaRepresentation
			(int i)
		{
			return changedObjectMetaRepresentations[i];
		}

		public virtual IList<ChangedObjectInfo
			> GetChanges()
		{
			return changes;
		}

		public virtual IList<NewNonNativeObjectAction
			> GetNewObjectMetaRepresentations()
		{
			return newObjectMetaRepresentations;
		}

		public virtual NewNonNativeObjectAction
			 GetNewObjectMetaRepresentation(int i)
		{
			return newObjectMetaRepresentations[i];
		}

		public virtual IList<object> GetNewObjects()
		{
			return newObjects;
		}

		public virtual int GetMaxObjectRecursionLevel()
		{
			return maxObjectRecursionLevel;
		}

		public virtual IList<ChangedAttribute
			> GetChangedAttributeActions()
		{
			return changedAttributeActions;
		}

		public virtual void SetChangedAttributeActions(IList<ChangedAttribute
			> changedAttributeActions)
		{
			this.changedAttributeActions = changedAttributeActions;
		}

		public virtual IList<SetAttributeToNullAction
			> GetAttributeToSetToNull()
		{
			return attributeToSetToNull;
		}

		public virtual void Clear()
		{
			changedObjectMetaRepresentations.Clear();
			attributeToSetToNull.Clear();
			alreadyCheckingObjects.Clear();
			newObjects.Clear();
			newObjectMetaRepresentations.Clear();
			changes.Clear();
			changedAttributeActions.Clear();
			arrayChanges.Clear();
			maxObjectRecursionLevel = 0;
			nbChanges = 0;
			supportInPlaceUpdate = false;
		}

		public virtual int GetNbChanges()
		{
			return nbChanges;
		}

		public override string ToString()
		{
			return nbChanges + " changes";
		}

		public virtual IList<ArrayModifyElement
			> GetArrayChanges()
		{
			return arrayChanges;
		}

		public virtual bool SupportInPlaceUpdate()
		{
			return supportInPlaceUpdate;
		}
	}
}
