using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000B0 RID: 176
	[HandlerCategory("vvPosClose"), HandlerName("StopLoss Abs")]
	public class StopLossAbs : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600066B RID: 1643 RVA: 0x0001D9B4 File Offset: 0x0001BBB4
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			pos.OpenMFE(barNum);
			return pos.get_EntryPrice() + (pos.get_IsLong() ? (-this.StopLoss) : this.StopLoss);
		}

		// Token: 0x17000239 RID: 569
		[HandlerParameter(true, "300", Min = "10", Max = "500", Step = "10", Name = "Стоп-лосс")]
		public double StopLoss
		{
			// Token: 0x06000669 RID: 1641 RVA: 0x0001D9A2 File Offset: 0x0001BBA2
			get;
			// Token: 0x0600066A RID: 1642 RVA: 0x0001D9AA File Offset: 0x0001BBAA
			set;
		}
	}
}
