using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F3 RID: 243
	[HandlerCategory("vvTrade"), HandlerName("Максимум позиции")]
	public class PosHighPrice : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600074C RID: 1868 RVA: 0x000207A4 File Offset: 0x0001E9A4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			return security.get_HighPrices()[pos.FindHighBar(barNum)];
		}
	}
}
