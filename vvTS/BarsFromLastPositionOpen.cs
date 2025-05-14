using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000EE RID: 238
	[HandlerCategory("vvTrade"), HandlerName("Баров с открытия посл. позиции")]
	public class BarsFromLastPositionOpen : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600073E RID: 1854 RVA: 0x000205F0 File Offset: 0x0001E7F0
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionActive = sec.get_Positions().GetLastPositionActive(barNum);
			IPosition lastPositionClosed = sec.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionActive == null && lastPositionClosed == null)
			{
				return 0.0;
			}
			int val = (lastPositionActive == null) ? 0 : lastPositionActive.get_EntryBarNum();
			int val2 = (lastPositionClosed == null) ? 0 : lastPositionClosed.get_EntryBarNum();
			int num = Math.Max(val, val2);
			return (double)((barNum > num) ? (barNum - num) : 0);
		}
	}
}
