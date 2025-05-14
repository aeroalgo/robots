using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000EC RID: 236
	[HandlerCategory("vvTrade"), HandlerName("Баров с закрытия посл. лонга")]
	public class BarsFromLastLongClosed12 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600073A RID: 1850 RVA: 0x00020580 File Offset: 0x0001E780
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastLongPositionClosed = sec.get_Positions().GetLastLongPositionClosed(barNum);
			if (lastLongPositionClosed == null)
			{
				return 0.0;
			}
			return (double)(barNum - lastLongPositionClosed.get_ExitBarNum());
		}
	}
}
