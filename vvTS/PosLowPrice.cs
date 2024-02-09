using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F4 RID: 244
	[HandlerCategory("vvTrade"), HandlerName("Минимум позиции")]
	public class PosLowPrice : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600074E RID: 1870 RVA: 0x000207E4 File Offset: 0x0001E9E4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			return security.get_LowPrices()[pos.FindLowBar(barNum)];
		}
	}
}
