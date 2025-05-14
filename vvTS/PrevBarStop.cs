using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000100 RID: 256
	[HandlerCategory("vvTrade"), HandlerName("Стоп последнего бара")]
	public class PrevBarStop : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000766 RID: 1894 RVA: 0x000209C8 File Offset: 0x0001EBC8
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return pos.GetStop(barNum);
		}
	}
}
