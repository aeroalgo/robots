using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F9 RID: 249
	[HandlerCategory("vvTrade"), HandlerName("Цена выхода")]
	public class ExitPrice : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000758 RID: 1880 RVA: 0x000208E0 File Offset: 0x0001EAE0
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return pos.get_ExitPrice();
		}
	}
}
