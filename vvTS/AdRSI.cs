using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200012A RID: 298
	[HandlerCategory("vvRSI"), HandlerName("adRSI")]
	public class AdRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008AE RID: 2222 RVA: 0x0002495C File Offset: 0x00022B5C
		public IList<double> Execute(IList<double> src)
		{
			return this.GenAdRSI(src, this.RSIperiod, this.Context);
		}

		// Token: 0x060008AD RID: 2221 RVA: 0x00024850 File Offset: 0x00022A50
		public IList<double> GenAdRSI(IList<double> src, int _RSIperiod, IContext context)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> data = context.GetData("rsi", new string[]
			{
				_RSIperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSIperiod));
			for (int i = 0; i < count; i++)
			{
				double num = Math.Abs(data[i] / 100.0 - 0.5) * 2.0;
				if (i <= _RSIperiod)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = array[i - 1] + num * (src[i] - array[i - 1]);
				}
			}
			return array;
		}

		// Token: 0x170002C7 RID: 711
		public IContext Context
		{
			// Token: 0x060008AF RID: 2223 RVA: 0x00024971 File Offset: 0x00022B71
			get;
			// Token: 0x060008B0 RID: 2224 RVA: 0x00024979 File Offset: 0x00022B79
			set;
		}

		// Token: 0x170002C6 RID: 710
		[HandlerParameter(true, "14", Min = "2", Max = "30", Step = "0")]
		public int RSIperiod
		{
			// Token: 0x060008AB RID: 2219 RVA: 0x00024822 File Offset: 0x00022A22
			get;
			// Token: 0x060008AC RID: 2220 RVA: 0x0002482A File Offset: 0x00022A2A
			set;
		}
	}
}
