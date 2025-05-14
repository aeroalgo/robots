using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000111 RID: 273
	[HandlerCategory("vvTrade"), HandlerName("Минимум за (перем. период)"), InputInfo(1, "Период"), InputInfo(0, "Данные")]
	public class LowestForXX : ITwoSourcesHandler, IDoubleInput0, IDoubleInput1, IDoubleReturns, IStreamHandler, IHandler, IContextUses
	{
		// Token: 0x060007A6 RID: 1958 RVA: 0x00021900 File Offset: 0x0001FB00
		public IList<double> Execute(IList<double> price, IList<double> period)
		{
			double[] array = new double[this.Context.get_BarsCount()];
			for (int i = 0; i < this.Context.get_BarsCount(); i++)
			{
				int currPeriod = Math.Max(Convert.ToInt32(period[i]), 1);
				IList<double> data = this.Context.GetData("MinForVarPeriod", new string[]
				{
					currPeriod.ToString(),
					price.GetHashCode().ToString()
				}, () => Series.Lowest(price, currPeriod));
				array[i] = data[i];
			}
			return array;
		}

		// Token: 0x1700026C RID: 620
		public IContext Context
		{
			// Token: 0x060007A5 RID: 1957 RVA: 0x000218CE File Offset: 0x0001FACE
			private get;
			// Token: 0x060007A4 RID: 1956 RVA: 0x000218C5 File Offset: 0x0001FAC5
			set;
		}
	}
}
