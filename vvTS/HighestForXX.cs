using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000110 RID: 272
	[HandlerCategory("vvTrade"), HandlerName("Максимум за (перем. период)"), InputInfo(0, "Данные"), InputInfo(1, "Период")]
	public class HighestForXX : ITwoSourcesHandler, IDoubleInput0, IDoubleInput1, IDoubleReturns, IStreamHandler, IHandler, IContextUses
	{
		// Token: 0x060007A2 RID: 1954 RVA: 0x000217F4 File Offset: 0x0001F9F4
		public IList<double> Execute(IList<double> price, IList<double> period)
		{
			double[] array = new double[this.Context.get_BarsCount()];
			for (int i = 0; i < this.Context.get_BarsCount(); i++)
			{
				int currPeriod = Math.Max(Convert.ToInt32(period[i]), 1);
				IList<double> data = this.Context.GetData("MaxForVarPeriod", new string[]
				{
					currPeriod.ToString(),
					price.GetHashCode().ToString()
				}, () => Series.Highest(price, currPeriod));
				array[i] = data[i];
			}
			return array;
		}

		// Token: 0x1700026B RID: 619
		public IContext Context
		{
			// Token: 0x060007A1 RID: 1953 RVA: 0x000217C2 File Offset: 0x0001F9C2
			private get;
			// Token: 0x060007A0 RID: 1952 RVA: 0x000217B9 File Offset: 0x0001F9B9
			set;
		}
	}
}
