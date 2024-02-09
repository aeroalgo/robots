using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000D2 RID: 210
	[HandlerCategory("vvTrade"), HandlerName("Пифагор")]
	public class Pifagor : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000700 RID: 1792 RVA: 0x0001F62C File Offset: 0x0001D82C
		public IList<double> Execute(IList<double> price)
		{
			double[] array = new double[price.Count];
			for (int i = 0; i < price.Count; i++)
			{
				string str = price[i].ToString();
				string str2 = this.strSum(str).ToString();
				int num = this.strSum(str2);
				if (num == 10)
				{
					array[i] = 1.0;
				}
				else
				{
					array[i] = Convert.ToDouble(this.strSum(num.ToString()));
				}
			}
			return array;
		}

		// Token: 0x060006FF RID: 1791 RVA: 0x0001F5F8 File Offset: 0x0001D7F8
		private int strSum(string str)
		{
			int num = 0;
			for (int i = 0; i < str.Length; i++)
			{
				string value = str.Substring(i, 1);
				num += Convert.ToInt32(value);
			}
			return num;
		}
	}
}
