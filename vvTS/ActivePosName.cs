using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000EA RID: 234
	[HandlerCategory("vvTrade"), HandlerName("Активна ли позиция\nс указанным именем?")]
	public class ActivePosName : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000736 RID: 1846 RVA: 0x000204BC File Offset: 0x0001E6BC
		public bool Execute(ISecurity sec, int barNum)
		{
			int activePositionCount = sec.get_Positions().get_ActivePositionCount();
			if (activePositionCount < 1)
			{
				return false;
			}
			IPositionsList positions = sec.get_Positions();
			bool result = false;
			foreach (IPosition current in positions)
			{
				if (current.IsActiveForbar(barNum) && string.CompareOrdinal(current.get_EntrySignalName(), this.EntryName) == 0)
				{
					result = true;
					break;
				}
			}
			return result;
		}

		// Token: 0x1700025E RID: 606
		[HandlerParameter(true, "", NotOptimized = true)]
		public string EntryName
		{
			// Token: 0x06000734 RID: 1844 RVA: 0x000204A8 File Offset: 0x0001E6A8
			get;
			// Token: 0x06000735 RID: 1845 RVA: 0x000204B0 File Offset: 0x0001E6B0
			set;
		}
	}
}
