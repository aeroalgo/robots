using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000EB RID: 235
	[HandlerCategory("vvTrade"), HandlerName("Баров с закрытия посл. позиции")]
	public class BarsFromLastPositionClosed12 : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000738 RID: 1848 RVA: 0x00020548 File Offset: 0x0001E748
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionClosed = sec.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed == null)
			{
				return 0.0;
			}
			return (double)(barNum - lastPositionClosed.get_ExitBarNum());
		}
	}
}
