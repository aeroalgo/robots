using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FD RID: 253
	[HandlerCategory("vvTrade"), HandlerName("Номер бара выхода из позиции (от инструм.)")]
	public class ExitBarNumFromSrc : IOneSourceHandler, IDoubleReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000760 RID: 1888 RVA: 0x00020970 File Offset: 0x0001EB70
		public double Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionClosed = sec.get_Positions().GetLastPositionClosed(barNum);
			if (lastPositionClosed == null)
			{
				return 0.0;
			}
			return (double)lastPositionClosed.get_ExitBarNum();
		}
	}
}
