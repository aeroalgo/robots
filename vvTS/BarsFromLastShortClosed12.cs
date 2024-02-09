using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000ED RID: 237
	[HandlerCategory("vvTrade"), HandlerName("Баров с закрытия посл. шорта")]
	public class BarsFromLastShortClosed12 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600073C RID: 1852 RVA: 0x000205B8 File Offset: 0x0001E7B8
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastShortPositionClosed = sec.get_Positions().GetLastShortPositionClosed(barNum);
			if (lastShortPositionClosed == null)
			{
				return 0.0;
			}
			return (double)(barNum - lastShortPositionClosed.get_ExitBarNum());
		}
	}
}
