using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F8 RID: 248
	[HandlerCategory("vvTrade"), HandlerName("Цена входа")]
	public class EntryPrice : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000756 RID: 1878 RVA: 0x000208C3 File Offset: 0x0001EAC3
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return pos.get_EntryPrice();
		}
	}
}
