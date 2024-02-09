using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000028 RID: 40
	[HandlerCategory("vvIndicators"), HandlerName("Force Index"), InputInfo(0, "Цена"), InputInfo(1, "Объём")]
	public class ForceIndex : BasePeriodIndicatorHandler, IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000173 RID: 371 RVA: 0x00006BF4 File Offset: 0x00004DF4
		public IList<double> Execute(IList<double> src1, IList<double> src2)
		{
			double[] array = new double[src1.Count];
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				base.get_Period().ToString(),
				src1.GetHashCode().ToString()
			}, () => Series.EMA(src1, this.get_Period()));
			for (int i = 1; i < src1.Count; i++)
			{
				array[i] = src2[i] * (data[i] - data[i - 1]);
			}
			return array;
		}

		// Token: 0x1700007C RID: 124
		public IContext Context
		{
			// Token: 0x06000174 RID: 372 RVA: 0x00006CAB File Offset: 0x00004EAB
			get;
			// Token: 0x06000175 RID: 373 RVA: 0x00006CB3 File Offset: 0x00004EB3
			set;
		}
	}
}
