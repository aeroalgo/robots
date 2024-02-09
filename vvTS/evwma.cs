using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000160 RID: 352
	[HandlerCategory("vvAverages"), HandlerName("evwma"), InputInfo(1, "Объём"), InputInfo(0, "Данные")]
	public class evwma : ITwoSourcesHandler, IDoubleInput0, IDoubleInput1, IDoubleReturns, IStreamHandler, IHandler, IContextUses
	{
		// Token: 0x06000B27 RID: 2855 RVA: 0x0002DC70 File Offset: 0x0002BE70
		public IList<double> Execute(IList<double> price, IList<double> volume)
		{
			return this.Context.GetData("evwma", new string[]
			{
				this.VolumePeriod.ToString(),
				price.GetHashCode().ToString()
			}, () => evwma.GenEVWMA(price, volume, this.VolumePeriod));
		}

		// Token: 0x06000B26 RID: 2854 RVA: 0x0002DB8C File Offset: 0x0002BD8C
		public static IList<double> GenEVWMA(IList<double> price, IList<double> volume, int _VolumePeriod)
		{
			double[] array = new double[price.Count];
			double[] array2 = new double[price.Count];
			array2[0] = volume[0];
			array[0] = price[0];
			for (int i = 1; i < price.Count; i++)
			{
				if (i < _VolumePeriod)
				{
					array2[i] = array2[i - 1] + volume[i];
					array[i] = volume[i] * price[i] / array2[i];
				}
				else
				{
					array2[i] = array2[i - 1] + volume[i] - volume[i - _VolumePeriod];
					array[i] = ((array2[i] - volume[i]) * array[i - 1] + volume[i] * price[i]) / array2[i];
				}
			}
			return array;
		}

		// Token: 0x170003AD RID: 941
		public IContext Context
		{
			// Token: 0x06000B25 RID: 2853 RVA: 0x0002DB81 File Offset: 0x0002BD81
			private get;
			// Token: 0x06000B24 RID: 2852 RVA: 0x0002DB78 File Offset: 0x0002BD78
			set;
		}

		// Token: 0x170003AC RID: 940
		[HandlerParameter(true, "30", Min = "10", Max = "100", Step = "5")]
		public int VolumePeriod
		{
			// Token: 0x06000B22 RID: 2850 RVA: 0x0002DB67 File Offset: 0x0002BD67
			get;
			// Token: 0x06000B23 RID: 2851 RVA: 0x0002DB6F File Offset: 0x0002BD6F
			set;
		}
	}
}
