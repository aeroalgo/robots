using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000E7 RID: 231
	[HandlerCategory("vvTrade"), HandlerName("Убыток ли в посл. сделке?")]
	public class LossInLastTrade : IBar2BoolHandler, IOneSourceHandler, IBooleanReturns, IValuesHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600072E RID: 1838 RVA: 0x00020318 File Offset: 0x0001E518
		public bool Execute(ISecurity sec, int barNum)
		{
			IPosition lastPositionClosed = sec.get_Positions().GetLastPositionClosed(barNum);
			return lastPositionClosed != null && lastPositionClosed.Profit() < 0.0;
		}
	}
}
