using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200007B RID: 123
	public abstract class BaseSen
	{
		// Token: 0x06000469 RID: 1129 RVA: 0x000172D4 File Offset: 0x000154D4
		private static double CalcMiddle(IList<Bar> src, int start, int finish)
		{
			double num = 3.4028234663852886E+38;
			double num2 = -3.4028234663852886E+38;
			for (int i = start; i <= finish; i++)
			{
				Bar bar = src[i];
				if (bar.get_High() > num2)
				{
					num2 = bar.get_High();
				}
				if (bar.get_Low() < num)
				{
					num = bar.get_Low();
				}
			}
			return 0.5 * (num2 + num);
		}

		// Token: 0x0600046A RID: 1130 RVA: 0x00017338 File Offset: 0x00015538
		private static double CalcSmoothMiddle(IList<Bar> src, int start, int finish)
		{
			double num = 0.0;
			for (int i = start; i <= finish; i++)
			{
				Bar bar = src[i];
				num += bar.get_High() + bar.get_Low() + bar.get_Close() + bar.get_Open();
			}
			return num / (double)(4 * (finish - start + 1));
		}

		// Token: 0x0600046B RID: 1131 RVA: 0x000173D0 File Offset: 0x000155D0
		public IList<double> Execute(ISecurity _sec)
		{
			IList<double> data = this.Context.GetData("hhv", new string[]
			{
				this.Period.ToString(),
				_sec.get_CacheName()
			}, () => Series.Highest(_sec.get_HighPrices(), this.Period));
			IList<double> data2 = this.Context.GetData("llv", new string[]
			{
				this.Period.ToString(),
				_sec.get_CacheName()
			}, () => Series.Lowest(_sec.get_LowPrices(), this.Period));
			double[] array = new double[data.Count];
			for (int i = 0; i < data.Count; i++)
			{
				array[i] = 0.5 * (data[i] + data2[i]);
			}
			return array;
		}

		// Token: 0x17000181 RID: 385
		public IContext Context
		{
			// Token: 0x06000467 RID: 1127 RVA: 0x000172C2 File Offset: 0x000154C2
			get;
			// Token: 0x06000468 RID: 1128 RVA: 0x000172CA File Offset: 0x000154CA
			set;
		}

		// Token: 0x17000180 RID: 384
		public virtual int Period
		{
			// Token: 0x06000465 RID: 1125 RVA: 0x000172B1 File Offset: 0x000154B1
			get;
			// Token: 0x06000466 RID: 1126 RVA: 0x000172B9 File Offset: 0x000154B9
			set;
		}
	}
}
